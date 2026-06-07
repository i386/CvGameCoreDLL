#include "CvGameCoreDLL.h"
#include "CvGameBridge.h"

#include "CvDLLInterfaceIFaceBase.h"
#include "CvDLLEngineIFaceBase.h"
#include "ThirdParty/parson/parson.h"

namespace
{
	static const DWORD PIPE_BUFFER_SIZE = 8192;
	static const char* DEFAULT_CONTROL_PIPE_NAME = "\\\\.\\pipe\\CvGameCoreDLL-Control";
	static const char* DEFAULT_CALLBACK_PIPE_NAME = "\\\\.\\pipe\\CvGameCoreDLL-Callbacks";

	struct BridgePipe
	{
		HANDLE hPipe;
		bool bConnected;
		CvString szName;
		CvString szReadBuffer;

		BridgePipe() : hPipe(INVALID_HANDLE_VALUE), bConnected(false) {}
	};

	static bool g_bEnabled = false;
	static unsigned int g_uiNextSeq = 1;
	static BridgePipe g_kControlPipe;
	static BridgePipe g_kCallbackPipe;

	void closePipe(BridgePipe& kPipe);

	void setStringFromEnv(CvString& szValue, const char* szEnvName, const char* szDefault)
	{
		char szBuffer[512];
		DWORD dwLength = GetEnvironmentVariableA(szEnvName, szBuffer, sizeof(szBuffer));
		szValue = (dwLength > 0 && dwLength < sizeof(szBuffer)) ? szBuffer : szDefault;
	}

	bool isEnvEnabled()
	{
		char szBuffer[32];
		DWORD dwLength = GetEnvironmentVariableA("CVGAME_BRIDGE", szBuffer, sizeof(szBuffer));
		return (dwLength > 0 && stricmp(szBuffer, "0") != 0 && stricmp(szBuffer, "false") != 0);
	}

	bool ensurePipe(BridgePipe& kPipe)
	{
		if (!g_bEnabled)
		{
			return false;
		}

		if (kPipe.hPipe != INVALID_HANDLE_VALUE)
		{
			return true;
		}

		kPipe.hPipe = CreateNamedPipeA(
			kPipe.szName.GetCString(),
			PIPE_ACCESS_DUPLEX,
			PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_NOWAIT,
			1,
			PIPE_BUFFER_SIZE,
			PIPE_BUFFER_SIZE,
			0,
			NULL);

		if (kPipe.hPipe == INVALID_HANDLE_VALUE)
		{
			OutputDebugString("CvGameBridge: failed to create named pipe\n");
			return false;
		}

		return true;
	}

	void disconnectPipe(BridgePipe& kPipe)
	{
		if (kPipe.hPipe != INVALID_HANDLE_VALUE)
		{
			FlushFileBuffers(kPipe.hPipe);
			DisconnectNamedPipe(kPipe.hPipe);
		}

		kPipe.bConnected = false;
		kPipe.szReadBuffer.clear();
	}

	void closePipe(BridgePipe& kPipe)
	{
		if (kPipe.hPipe != INVALID_HANDLE_VALUE)
		{
			disconnectPipe(kPipe);
			CloseHandle(kPipe.hPipe);
			kPipe.hPipe = INVALID_HANDLE_VALUE;
		}
	}

	bool writeLine(BridgePipe& kPipe, const CvString& szLine)
	{
		if (!kPipe.bConnected || kPipe.hPipe == INVALID_HANDLE_VALUE)
		{
			return false;
		}

		CvString szOut = szLine;
		if (szOut.empty() || szOut[szOut.size() - 1] != '\n')
		{
			szOut += "\n";
		}

		DWORD dwWritten = 0;
		BOOL bOk = WriteFile(kPipe.hPipe, szOut.GetCString(), (DWORD)szOut.size(), &dwWritten, NULL);
		if (!bOk)
		{
			DWORD dwError = GetLastError();
			if (dwError == ERROR_BROKEN_PIPE || dwError == ERROR_NO_DATA || dwError == ERROR_PIPE_NOT_CONNECTED)
			{
				disconnectPipe(kPipe);
			}
			return false;
		}

		return true;
	}

	CvString serializeAndFree(JSON_Value* pValue)
	{
		CvString szResult;
		char* szSerialized = json_serialize_to_string(pValue);
		if (szSerialized != NULL)
		{
			szResult = szSerialized;
			json_free_serialized_string(szSerialized);
		}
		json_value_free(pValue);
		return szResult;
	}

	JSON_Value* makeBaseMessage(const char* szType)
	{
		JSON_Value* pValue = json_value_init_object();
		JSON_Object* pObject = json_value_get_object(pValue);
		json_object_set_string(pObject, "type", szType);
		return pValue;
	}

	CvString makeReply(int iId, bool bOk)
	{
		JSON_Value* pValue = makeBaseMessage("reply");
		JSON_Object* pObject = json_value_get_object(pValue);
		json_object_set_number(pObject, "id", iId);
		json_object_set_boolean(pObject, "ok", bOk ? 1 : 0);
		return serializeAndFree(pValue);
	}

	CvString makeErrorReply(int iId, const char* szCode, const char* szMessage)
	{
		JSON_Value* pValue = makeBaseMessage("reply");
		JSON_Object* pObject = json_value_get_object(pValue);
		JSON_Value* pErrorValue = json_value_init_object();
		JSON_Object* pError = json_value_get_object(pErrorValue);
		json_object_set_number(pObject, "id", iId);
		json_object_set_boolean(pObject, "ok", 0);
		json_object_set_string(pError, "code", szCode);
		json_object_set_string(pError, "message", szMessage);
		json_object_set_value(pObject, "error", pErrorValue);
		return serializeAndFree(pValue);
	}

	bool getInt(JSON_Object* pObject, const char* szName, int& iValue)
	{
		JSON_Value* pValue = json_object_get_value(pObject, szName);
		if (pValue == NULL || json_value_get_type(pValue) != JSONNumber)
		{
			return false;
		}
		iValue = (int)json_value_get_number(pValue);
		return true;
	}

	int getInfoTypeFromValue(JSON_Value* pValue)
	{
		if (pValue == NULL)
		{
			return -1;
		}
		if (json_value_get_type(pValue) == JSONNumber)
		{
			return (int)json_value_get_number(pValue);
		}
		if (json_value_get_type(pValue) == JSONString)
		{
			return GC.getInfoTypeForString(json_value_get_string(pValue), true);
		}
		return -1;
	}

	bool validPlayer(int iPlayer)
	{
		return (iPlayer >= 0 && iPlayer < GC.getMAX_PLAYERS());
	}

	bool canMutate()
	{
		return !GC.getGameINLINE().isGameMultiPlayer();
	}

	void markGameDataDirty()
	{
		if (gDLL != NULL)
		{
			gDLL->getInterfaceIFace()->setDirty(GameData_DIRTY_BIT, true);
			gDLL->getInterfaceIFace()->setDirty(Score_DIRTY_BIT, true);
			gDLL->getInterfaceIFace()->setDirty(CityInfo_DIRTY_BIT, true);
			gDLL->getInterfaceIFace()->setDirty(UnitInfo_DIRTY_BIT, true);
			gDLL->getEngineIFace()->SetDirty(MinimapTexture_DIRTY_BIT, true);
		}
	}

	void setReplyResultString(CvString& szReply, int iId, const char* szName, const char* szValue)
	{
		JSON_Value* pValue = makeBaseMessage("reply");
		JSON_Object* pObject = json_value_get_object(pValue);
		JSON_Value* pResultValue = json_value_init_object();
		JSON_Object* pResult = json_value_get_object(pResultValue);
		json_object_set_number(pObject, "id", iId);
		json_object_set_boolean(pObject, "ok", 1);
		json_object_set_string(pResult, szName, szValue);
		json_object_set_value(pObject, "result", pResultValue);
		szReply = serializeAndFree(pValue);
	}

	void setReplyResultInt(CvString& szReply, int iId, const char* szName, int iValue)
	{
		JSON_Value* pValue = makeBaseMessage("reply");
		JSON_Object* pObject = json_value_get_object(pValue);
		JSON_Value* pResultValue = json_value_init_object();
		JSON_Object* pResult = json_value_get_object(pResultValue);
		json_object_set_number(pObject, "id", iId);
		json_object_set_boolean(pObject, "ok", 1);
		json_object_set_number(pResult, szName, iValue);
		json_object_set_value(pObject, "result", pResultValue);
		szReply = serializeAndFree(pValue);
	}

	CvString handleQuery(int iId, const char* szName, JSON_Object* pArgs)
	{
		if (strcmp(szName, "get_game_turn") == 0)
		{
			CvString szReply;
			setReplyResultInt(szReply, iId, "turn", GC.getGameINLINE().getGameTurn());
			return szReply;
		}

		if (strcmp(szName, "get_player_gold") == 0)
		{
			int iPlayer = -1;
			if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
			{
				return makeErrorReply(iId, "bad_player", "player is missing or out of range");
			}
			CvString szReply;
			setReplyResultInt(szReply, iId, "gold", GET_PLAYER((PlayerTypes)iPlayer).getGold());
			return szReply;
		}

		if (strcmp(szName, "get_mod_state") == 0)
		{
			CvString szReply;
			setReplyResultString(szReply, iId, "json", GC.getGameINLINE().getBridgeModState().GetCString());
			return szReply;
		}

		return makeErrorReply(iId, "unknown_query", "query name is not supported");
	}

	CvString handleSetPlayerGold(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iValue = 0;
		if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue))
		{
			return makeErrorReply(iId, "bad_value", "value is missing");
		}

		GET_PLAYER((PlayerTypes)iPlayer).setGold(iValue);
		markGameDataDirty();

		CvString szReply;
		setReplyResultInt(szReply, iId, "gold", GET_PLAYER((PlayerTypes)iPlayer).getGold());
		return szReply;
	}

	CvString handleSpawnUnit(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iX = -1;
		int iY = -1;
		if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "x", iX) || !getInt(pArgs, "y", iY) || GC.getMapINLINE().plot(iX, iY) == NULL)
		{
			return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
		}

		int iUnit = getInfoTypeFromValue(json_object_get_value(pArgs, "unit_type"));
		if (iUnit < 0 || iUnit >= GC.getNumUnitInfos())
		{
			return makeErrorReply(iId, "bad_unit_type", "unit_type is missing or out of range");
		}

		int iUnitAI = NO_UNITAI;
		JSON_Value* pUnitAI = json_object_get_value(pArgs, "unit_ai");
		if (pUnitAI != NULL)
		{
			iUnitAI = getInfoTypeFromValue(pUnitAI);
			if (iUnitAI < NO_UNITAI || iUnitAI >= (int)GC.getUnitAIInfo().size())
			{
				return makeErrorReply(iId, "bad_unit_ai", "unit_ai is out of range");
			}
		}

		CvUnit* pUnit = GET_PLAYER((PlayerTypes)iPlayer).initUnit((UnitTypes)iUnit, iX, iY, (UnitAITypes)iUnitAI);
		if (pUnit == NULL)
		{
			return makeErrorReply(iId, "spawn_failed", "unit could not be created");
		}

		markGameDataDirty();

		JSON_Value* pValue = makeBaseMessage("reply");
		JSON_Object* pObject = json_value_get_object(pValue);
		JSON_Value* pResultValue = json_value_init_object();
		JSON_Object* pResult = json_value_get_object(pResultValue);
		json_object_set_number(pObject, "id", iId);
		json_object_set_boolean(pObject, "ok", 1);
		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_number(pResult, "unit", pUnit->getID());
		json_object_set_number(pResult, "x", pUnit->getX_INLINE());
		json_object_set_number(pResult, "y", pUnit->getY_INLINE());
		json_object_set_value(pObject, "result", pResultValue);
		return serializeAndFree(pValue);
	}

	CvString handleSetModState(int iId, JSON_Object* pArgs)
	{
		const char* szJson = json_object_get_string(pArgs, "json");
		if (szJson == NULL)
		{
			return makeErrorReply(iId, "bad_mod_state", "json string is missing");
		}

		GC.getGameINLINE().setBridgeModState(szJson);

		CvString szReply;
		setReplyResultInt(szReply, iId, "bytes", (int)strlen(szJson));
		return szReply;
	}

	CvString handleCommand(int iId, const char* szName, JSON_Object* pArgs)
	{
		if (!canMutate())
		{
			return makeErrorReply(iId, "multiplayer_read_only", "commands are disabled in multiplayer");
		}

		if (strcmp(szName, "set_player_gold") == 0)
		{
			return handleSetPlayerGold(iId, pArgs);
		}
		if (strcmp(szName, "spawn_unit") == 0)
		{
			return handleSpawnUnit(iId, pArgs);
		}
		if (strcmp(szName, "set_mod_state") == 0)
		{
			return handleSetModState(iId, pArgs);
		}

		return makeErrorReply(iId, "unknown_command", "command name is not supported");
	}

	void handleControlLine(const CvString& szLine)
	{
		JSON_Value* pValue = json_parse_string(szLine.GetCString());
		if (pValue == NULL || json_value_get_type(pValue) != JSONObject)
		{
			writeLine(g_kControlPipe, makeErrorReply(0, "bad_json", "message is not a JSON object"));
			json_value_free(pValue);
			return;
		}

		JSON_Object* pObject = json_value_get_object(pValue);
		const char* szType = json_object_get_string(pObject, "type");
		const char* szName = json_object_get_string(pObject, "name");
		int iId = (int)json_object_get_number(pObject, "id");
		JSON_Object* pArgs = json_object_get_object(pObject, "args");
		JSON_Value* pEmptyArgsValue = NULL;

		if (szType == NULL || szName == NULL || iId == 0)
		{
			writeLine(g_kControlPipe, makeErrorReply(iId, "bad_message", "type, id, or name is missing"));
			json_value_free(pValue);
			return;
		}
		if (pArgs == NULL)
		{
			pEmptyArgsValue = json_value_init_object();
			pArgs = json_value_get_object(pEmptyArgsValue);
		}

		CvString szReply;
		if (strcmp(szType, "query") == 0)
		{
			szReply = handleQuery(iId, szName, pArgs);
		}
		else if (strcmp(szType, "command") == 0)
		{
			szReply = handleCommand(iId, szName, pArgs);
		}
		else
		{
			szReply = makeErrorReply(iId, "bad_type", "only query and command are accepted on control pipe");
		}

		writeLine(g_kControlPipe, szReply);
		json_value_free(pEmptyArgsValue);
		json_value_free(pValue);
	}

	void processReadBuffer(BridgePipe& kPipe, bool bControl)
	{
		for (;;)
		{
			std::string::size_type iPos = kPipe.szReadBuffer.find_first_of("\r\n");
			if (iPos == CvString::npos)
			{
				break;
			}

			CvString szLine = kPipe.szReadBuffer.substr(0, iPos);
			kPipe.szReadBuffer.erase(0, iPos + 1);
			while (!kPipe.szReadBuffer.empty() && (kPipe.szReadBuffer[0] == '\r' || kPipe.szReadBuffer[0] == '\n'))
			{
				kPipe.szReadBuffer.erase(0, 1);
			}

			if (!szLine.empty() && bControl)
			{
				handleControlLine(szLine);
			}
		}

		if (kPipe.szReadBuffer.size() > PIPE_BUFFER_SIZE)
		{
			kPipe.szReadBuffer.clear();
			if (bControl)
			{
				writeLine(kPipe, makeErrorReply(0, "line_too_long", "message exceeded bridge buffer"));
			}
		}
	}

	void pollPipe(BridgePipe& kPipe, bool bControl)
	{
		if (!ensurePipe(kPipe))
		{
			return;
		}

		if (!kPipe.bConnected)
		{
			BOOL bConnected = ConnectNamedPipe(kPipe.hPipe, NULL);
			DWORD dwError = bConnected ? ERROR_SUCCESS : GetLastError();
			if (bConnected || dwError == ERROR_PIPE_CONNECTED)
			{
				kPipe.bConnected = true;
				if (bControl)
				{
					JSON_Value* pValue = makeBaseMessage("hello");
					JSON_Object* pObject = json_value_get_object(pValue);
					JSON_Value* pCapsValue = json_value_init_array();
					JSON_Array* pCaps = json_value_get_array(pCapsValue);
					json_object_set_number(pObject, "protocol", 1);
					json_object_set_string(pObject, "side", "dll");
					json_array_append_string(pCaps, "events");
					json_array_append_string(pCaps, "queries");
					json_array_append_string(pCaps, "commands");
					json_array_append_string(pCaps, "callbacks");
					json_array_append_string(pCaps, "mod_state");
					json_object_set_value(pObject, "capabilities", pCapsValue);
					writeLine(kPipe, serializeAndFree(pValue));
				}
			}
			else
			{
				return;
			}
		}

		for (;;)
		{
			char szBuffer[PIPE_BUFFER_SIZE + 1];
			DWORD dwRead = 0;
			BOOL bOk = ReadFile(kPipe.hPipe, szBuffer, PIPE_BUFFER_SIZE, &dwRead, NULL);
			if (!bOk)
			{
				DWORD dwError = GetLastError();
				if (dwError == ERROR_MORE_DATA && dwRead > 0)
				{
					szBuffer[dwRead] = 0;
					kPipe.szReadBuffer += szBuffer;
					continue;
				}
				if (dwError == ERROR_BROKEN_PIPE || dwError == ERROR_PIPE_NOT_CONNECTED)
				{
					disconnectPipe(kPipe);
				}
				break;
			}

			if (dwRead == 0)
			{
				break;
			}

			szBuffer[dwRead] = 0;
			kPipe.szReadBuffer += szBuffer;
			processReadBuffer(kPipe, bControl);
			if (!kPipe.bConnected)
			{
				break;
			}
		}
	}

	void sendNamedMessage(BridgePipe& kPipe, const char* szType, const char* szName, const char* szArgsJson)
	{
		if (!g_bEnabled || !kPipe.bConnected || szName == NULL)
		{
			return;
		}

		JSON_Value* pValue = makeBaseMessage(szType);
		JSON_Object* pObject = json_value_get_object(pValue);
		json_object_set_number(pObject, "seq", g_uiNextSeq++);
		json_object_set_string(pObject, "name", szName);

		if (szArgsJson != NULL && szArgsJson[0] != 0)
		{
			JSON_Value* pArgsValue = json_parse_string(szArgsJson);
			if (pArgsValue != NULL && json_value_get_type(pArgsValue) == JSONObject)
			{
				json_object_set_value(pObject, "args", pArgsValue);
			}
			else
			{
				json_value_free(pArgsValue);
			}
		}

		writeLine(kPipe, serializeAndFree(pValue));
	}
}

void CvGameBridge::init()
{
	if (g_bEnabled)
	{
		return;
	}

	g_bEnabled = isEnvEnabled();
	if (!g_bEnabled)
	{
		return;
	}

	CvString szPrefix;
	setStringFromEnv(szPrefix, "CVGAME_BRIDGE_PIPE_PREFIX", "");
	if (!szPrefix.empty())
	{
		g_kControlPipe.szName.Format("\\\\.\\pipe\\%s-Control", szPrefix.GetCString());
		g_kCallbackPipe.szName.Format("\\\\.\\pipe\\%s-Callbacks", szPrefix.GetCString());
	}
	else
	{
		setStringFromEnv(g_kControlPipe.szName, "CVGAME_BRIDGE_CONTROL_PIPE", DEFAULT_CONTROL_PIPE_NAME);
		setStringFromEnv(g_kCallbackPipe.szName, "CVGAME_BRIDGE_CALLBACK_PIPE", DEFAULT_CALLBACK_PIPE_NAME);
	}

	ensurePipe(g_kControlPipe);
	ensurePipe(g_kCallbackPipe);
	OutputDebugString("CvGameBridge: enabled\n");
}

void CvGameBridge::shutdown()
{
	closePipe(g_kControlPipe);
	closePipe(g_kCallbackPipe);
	g_bEnabled = false;
}

void CvGameBridge::poll()
{
	pollPipe(g_kControlPipe, true);
	pollPipe(g_kCallbackPipe, false);
}

bool CvGameBridge::isEnabled()
{
	return g_bEnabled;
}

void CvGameBridge::sendEvent(const char* szName, const char* szArgsJson)
{
	sendNamedMessage(g_kControlPipe, "event", szName, szArgsJson);
}

void CvGameBridge::sendCallbackMirror(const char* szName, const char* szArgsJson)
{
	sendNamedMessage(g_kCallbackPipe, "callback_mirror", szName, szArgsJson);
}
