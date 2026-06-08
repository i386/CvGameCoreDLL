#include "CvGameCoreDLL.h"
#include "CvGameBridge.h"

#include "CvDLLInterfaceIFaceBase.h"
#include "CvDLLEngineIFaceBase.h"
#include "ThirdParty/parson/parson.h"

namespace
{
	static const DWORD PIPE_BUFFER_SIZE = 8192;
	static const DWORD DEFAULT_CALLBACK_TIMEOUT_MS = 50;
	static const DWORD COMPANION_SHUTDOWN_TIMEOUT_MS = 2000;
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
	static unsigned int g_uiNextCallbackId = 1;
	static BridgePipe g_kControlPipe;
	static BridgePipe g_kCallbackPipe;
	static PROCESS_INFORMATION g_kCompanionProcessInfo;

	void closePipe(BridgePipe& kPipe);
	void pollPipe(BridgePipe& kPipe, bool bControl);
	void stopCompanionProcess();

	void setStringFromEnv(CvString& szValue, const char* szEnvName, const char* szDefault)
	{
		char szBuffer[512];
		DWORD dwLength = GetEnvironmentVariableA(szEnvName, szBuffer, sizeof(szBuffer));
		szValue = (dwLength > 0 && dwLength < sizeof(szBuffer)) ? szBuffer : szDefault;
	}

	bool isEnvTruthy(const char* szEnvName)
	{
		char szBuffer[32];
		DWORD dwLength = GetEnvironmentVariableA(szEnvName, szBuffer, sizeof(szBuffer));
		return (dwLength > 0 && stricmp(szBuffer, "0") != 0 && stricmp(szBuffer, "false") != 0);
	}

	bool isEnvEnabled()
	{
		return isEnvTruthy("CVGAME_BRIDGE");
	}

	CvString getParentDirectory(const CvString& szPath)
	{
		size_t iSlash = szPath.find_last_of("\\/");
		if (iSlash == CvString::npos)
		{
			return "";
		}
		return szPath.substr(0, iSlash);
	}

	bool fileExists(const CvString& szPath)
	{
		DWORD dwAttributes = GetFileAttributesA(szPath.GetCString());
		return (dwAttributes != INVALID_FILE_ATTRIBUTES && (dwAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0);
	}

	bool getDllDirectory(CvString& szDirectory)
	{
		char szModulePath[MAX_PATH];
		HMODULE hModule = GetModuleHandleA("CvGameCoreDLL.dll");
		DWORD dwLength = GetModuleFileNameA(hModule, szModulePath, MAX_PATH);
		if (dwLength == 0 || dwLength >= MAX_PATH)
		{
			return false;
		}

		szDirectory = getParentDirectory(szModulePath);
		return !szDirectory.empty();
	}

	CvString quoteCommandArgument(const CvString& szArgument)
	{
		CvString szQuoted = "\"";
		for (int iI = 0; iI < (int)szArgument.length(); ++iI)
		{
			if (szArgument[iI] == '"')
			{
				szQuoted += "\\\"";
			}
			else
			{
				szQuoted += szArgument[iI];
			}
		}
		szQuoted += "\"";
		return szQuoted;
	}

	bool findCompanionExe(CvString& szExePath)
	{
		setStringFromEnv(szExePath, "CVGAME_BRIDGE_COMPANION_EXE", "");
		if (!szExePath.empty())
		{
			return fileExists(szExePath);
		}

		CvString szDllDir;
		if (!getDllDirectory(szDllDir))
		{
			return false;
		}

		CvString aszCandidates[] =
		{
			szDllDir + "\\CvGameBridgeCompanion.exe",
			szDllDir + "\\AgesBeyondCompanion.exe",
			szDllDir + "\\..\\Companion\\CvGameBridgeCompanion.exe",
			szDllDir + "\\..\\Companion\\AgesBeyondCompanion.exe",
			szDllDir + "\\..\\CvGameBridgeCompanion.exe",
			szDllDir + "\\..\\AgesBeyondCompanion.exe"
		};

		for (int iI = 0; iI < 6; ++iI)
		{
			if (fileExists(aszCandidates[iI]))
			{
				szExePath = aszCandidates[iI];
				return true;
			}
		}

		return false;
	}

	void startCompanionProcess()
	{
		if (!isEnvTruthy("CVGAME_BRIDGE_AUTOLAUNCH") || g_kCompanionProcessInfo.hProcess != NULL)
		{
			return;
		}

		CvString szExePath;
		if (!findCompanionExe(szExePath))
		{
			OutputDebugString("CvGameBridge: companion autolaunch requested, but executable was not found\n");
			return;
		}

		CvString szCommandLine = quoteCommandArgument(szExePath);
		CvString szExtraArgs;
		setStringFromEnv(szExtraArgs, "CVGAME_BRIDGE_COMPANION_ARGS", "");
		if (!szExtraArgs.empty())
		{
			szCommandLine += " ";
			szCommandLine += szExtraArgs;
		}

		STARTUPINFOA kStartupInfo;
		ZeroMemory(&kStartupInfo, sizeof(kStartupInfo));
		kStartupInfo.cb = sizeof(kStartupInfo);
		kStartupInfo.dwFlags = STARTF_USESHOWWINDOW;
		kStartupInfo.wShowWindow = SW_HIDE;

		ZeroMemory(&g_kCompanionProcessInfo, sizeof(g_kCompanionProcessInfo));

		char* szMutableCommandLine = new char[szCommandLine.length() + 1];
		strcpy(szMutableCommandLine, szCommandLine.GetCString());

		CvString szWorkingDirectory = getParentDirectory(szExePath);
		BOOL bStarted = CreateProcessA(
			szExePath.GetCString(),
			szMutableCommandLine,
			NULL,
			NULL,
			FALSE,
			CREATE_NO_WINDOW,
			NULL,
			szWorkingDirectory.empty() ? NULL : szWorkingDirectory.GetCString(),
			&kStartupInfo,
			&g_kCompanionProcessInfo);

		delete[] szMutableCommandLine;

		if (!bStarted)
		{
			ZeroMemory(&g_kCompanionProcessInfo, sizeof(g_kCompanionProcessInfo));
			OutputDebugString("CvGameBridge: failed to launch companion process\n");
			return;
		}

		OutputDebugString("CvGameBridge: launched companion process\n");
	}

	void stopCompanionProcess()
	{
		if (g_kCompanionProcessInfo.hProcess != NULL)
		{
			if (WaitForSingleObject(g_kCompanionProcessInfo.hProcess, COMPANION_SHUTDOWN_TIMEOUT_MS) == WAIT_TIMEOUT)
			{
				TerminateProcess(g_kCompanionProcessInfo.hProcess, 0);
			}
			CloseHandle(g_kCompanionProcessInfo.hProcess);
			g_kCompanionProcessInfo.hProcess = NULL;
		}

		if (g_kCompanionProcessInfo.hThread != NULL)
		{
			CloseHandle(g_kCompanionProcessInfo.hThread);
			g_kCompanionProcessInfo.hThread = NULL;
		}
	}

	DWORD getCallbackTimeoutMs()
	{
		char szBuffer[32];
		DWORD dwLength = GetEnvironmentVariableA("CVGAME_BRIDGE_CALLBACK_TIMEOUT_MS", szBuffer, sizeof(szBuffer));
		if (dwLength > 0 && dwLength < sizeof(szBuffer))
		{
			int iValue = atoi(szBuffer);
			if (iValue > 0)
			{
				return (DWORD)iValue;
			}
		}
		return DEFAULT_CALLBACK_TIMEOUT_MS;
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

	bool readAvailable(BridgePipe& kPipe)
	{
		if (!kPipe.bConnected || kPipe.hPipe == INVALID_HANDLE_VALUE)
		{
			return false;
		}

		bool bReadAny = false;
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
					bReadAny = true;
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
			bReadAny = true;
		}
		return bReadAny;
	}

	bool popBufferedLine(BridgePipe& kPipe, CvString& szLine)
	{
		std::string::size_type iPos = kPipe.szReadBuffer.find_first_of("\r\n");
		if (iPos == CvString::npos)
		{
			return false;
		}

		szLine = kPipe.szReadBuffer.substr(0, iPos);
		kPipe.szReadBuffer.erase(0, iPos + 1);
		while (!kPipe.szReadBuffer.empty() && (kPipe.szReadBuffer[0] == '\r' || kPipe.szReadBuffer[0] == '\n'))
		{
			kPipe.szReadBuffer.erase(0, 1);
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
			if (pValue == NULL || json_value_get_type(pValue) != JSONBoolean)
			{
				return false;
			}
			iValue = json_value_get_boolean(pValue) ? 1 : 0;
			return true;
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

	int getOrderTypeFromValue(JSON_Value* pValue)
	{
		const char* szOrder = NULL;
		if (pValue == NULL)
		{
			return NO_ORDER;
		}
		if (json_value_get_type(pValue) == JSONNumber)
		{
			return (int)json_value_get_number(pValue);
		}
		if (json_value_get_type(pValue) != JSONString)
		{
			return NO_ORDER;
		}

		szOrder = json_value_get_string(pValue);
		if (stricmp(szOrder, "train") == 0 || stricmp(szOrder, "ORDER_TRAIN") == 0)
		{
			return ORDER_TRAIN;
		}
		if (stricmp(szOrder, "construct") == 0 || stricmp(szOrder, "ORDER_CONSTRUCT") == 0)
		{
			return ORDER_CONSTRUCT;
		}
		if (stricmp(szOrder, "create") == 0 || stricmp(szOrder, "ORDER_CREATE") == 0)
		{
			return ORDER_CREATE;
		}
		if (stricmp(szOrder, "maintain") == 0 || stricmp(szOrder, "ORDER_MAINTAIN") == 0)
		{
			return ORDER_MAINTAIN;
		}
		return NO_ORDER;
	}

	int getWarPlanTypeFromValue(JSON_Value* pValue)
	{
		const char* szWarPlan = NULL;
		if (pValue == NULL)
		{
			return NO_WARPLAN;
		}
		if (json_value_get_type(pValue) == JSONNumber)
		{
			return (int)json_value_get_number(pValue);
		}
		if (json_value_get_type(pValue) != JSONString)
		{
			return -2;
		}

		szWarPlan = json_value_get_string(pValue);
		if (stricmp(szWarPlan, "none") == 0 || stricmp(szWarPlan, "NO_WARPLAN") == 0)
		{
			return NO_WARPLAN;
		}
		if (stricmp(szWarPlan, "attacked_recent") == 0 || stricmp(szWarPlan, "WARPLAN_ATTACKED_RECENT") == 0)
		{
			return WARPLAN_ATTACKED_RECENT;
		}
		if (stricmp(szWarPlan, "attacked") == 0 || stricmp(szWarPlan, "WARPLAN_ATTACKED") == 0)
		{
			return WARPLAN_ATTACKED;
		}
		if (stricmp(szWarPlan, "preparing_limited") == 0 || stricmp(szWarPlan, "WARPLAN_PREPARING_LIMITED") == 0)
		{
			return WARPLAN_PREPARING_LIMITED;
		}
		if (stricmp(szWarPlan, "preparing_total") == 0 || stricmp(szWarPlan, "WARPLAN_PREPARING_TOTAL") == 0)
		{
			return WARPLAN_PREPARING_TOTAL;
		}
		if (stricmp(szWarPlan, "limited") == 0 || stricmp(szWarPlan, "WARPLAN_LIMITED") == 0)
		{
			return WARPLAN_LIMITED;
		}
		if (stricmp(szWarPlan, "total") == 0 || stricmp(szWarPlan, "WARPLAN_TOTAL") == 0)
		{
			return WARPLAN_TOTAL;
		}
		if (stricmp(szWarPlan, "dogpile") == 0 || stricmp(szWarPlan, "WARPLAN_DOGPILE") == 0)
		{
			return WARPLAN_DOGPILE;
		}
		return -2;
	}

	int getGameStateTypeFromValue(JSON_Value* pValue)
	{
		const char* szGameState = NULL;
		if (pValue == NULL)
		{
			return -1;
		}
		if (json_value_get_type(pValue) == JSONNumber)
		{
			return (int)json_value_get_number(pValue);
		}
		if (json_value_get_type(pValue) != JSONString)
		{
			return -1;
		}

		szGameState = json_value_get_string(pValue);
		if (stricmp(szGameState, "on") == 0 || stricmp(szGameState, "GAMESTATE_ON") == 0)
		{
			return GAMESTATE_ON;
		}
		if (stricmp(szGameState, "over") == 0 || stricmp(szGameState, "GAMESTATE_OVER") == 0)
		{
			return GAMESTATE_OVER;
		}
		if (stricmp(szGameState, "extended") == 0 || stricmp(szGameState, "GAMESTATE_EXTENDED") == 0)
		{
			return GAMESTATE_EXTENDED;
		}
		return -1;
	}

	int getCommerceTypeFromValue(JSON_Value* pValue)
	{
		const char* szCommerce = NULL;
		if (pValue == NULL)
		{
			return -1;
		}
		if (json_value_get_type(pValue) == JSONNumber)
		{
			return (int)json_value_get_number(pValue);
		}
		if (json_value_get_type(pValue) != JSONString)
		{
			return -1;
		}

		szCommerce = json_value_get_string(pValue);
		if (stricmp(szCommerce, "gold") == 0 || stricmp(szCommerce, "COMMERCE_GOLD") == 0)
		{
			return COMMERCE_GOLD;
		}
		if (stricmp(szCommerce, "research") == 0 || stricmp(szCommerce, "COMMERCE_RESEARCH") == 0)
		{
			return COMMERCE_RESEARCH;
		}
		if (stricmp(szCommerce, "culture") == 0 || stricmp(szCommerce, "COMMERCE_CULTURE") == 0)
		{
			return COMMERCE_CULTURE;
		}
		if (stricmp(szCommerce, "espionage") == 0 || stricmp(szCommerce, "COMMERCE_ESPIONAGE") == 0)
		{
			return COMMERCE_ESPIONAGE;
		}
		return -1;
	}

	const char* getCanonicalInfoKind(const char* szKind)
	{
		if (szKind == NULL)
		{
			return NULL;
		}
		if (stricmp(szKind, "unit") == 0) return "unit";
		if (stricmp(szKind, "unit_ai") == 0) return "unit_ai";
		if (stricmp(szKind, "building") == 0) return "building";
		if (stricmp(szKind, "building_class") == 0) return "building_class";
		if (stricmp(szKind, "project") == 0) return "project";
		if (stricmp(szKind, "process") == 0) return "process";
		if (stricmp(szKind, "terrain") == 0) return "terrain";
		if (stricmp(szKind, "feature") == 0) return "feature";
		if (stricmp(szKind, "bonus") == 0) return "bonus";
		if (stricmp(szKind, "improvement") == 0) return "improvement";
		if (stricmp(szKind, "route") == 0) return "route";
		if (stricmp(szKind, "promotion") == 0) return "promotion";
		if (stricmp(szKind, "tech") == 0) return "tech";
		if (stricmp(szKind, "civic") == 0) return "civic";
		if (stricmp(szKind, "civic_option") == 0) return "civic_option";
		if (stricmp(szKind, "religion") == 0) return "religion";
		if (stricmp(szKind, "corporation") == 0) return "corporation";
		if (stricmp(szKind, "victory") == 0) return "victory";
		if (stricmp(szKind, "game_option") == 0) return "game_option";
		if (stricmp(szKind, "multiplayer_option") == 0 || stricmp(szKind, "mp_option") == 0) return "multiplayer_option";
		if (stricmp(szKind, "force_control") == 0) return "force_control";
		if (stricmp(szKind, "era") == 0) return "era";
		if (stricmp(szKind, "leader") == 0 || stricmp(szKind, "leader_head") == 0) return "leader";
		if (stricmp(szKind, "civilization") == 0) return "civilization";
		if (stricmp(szKind, "handicap") == 0) return "handicap";
		if (stricmp(szKind, "game_speed") == 0) return "game_speed";
		if (stricmp(szKind, "hurry") == 0) return "hurry";
		if (stricmp(szKind, "build") == 0) return "build";
		if (stricmp(szKind, "goody") == 0) return "goody";
		if (stricmp(szKind, "mission") == 0) return "mission";
		if (stricmp(szKind, "espionage_mission") == 0) return "espionage_mission";
		if (stricmp(szKind, "specialist") == 0) return "specialist";
		if (stricmp(szKind, "unit_class") == 0) return "unit_class";
		if (stricmp(szKind, "unit_combat") == 0) return "unit_combat";
		if (stricmp(szKind, "player_option") == 0) return "player_option";
		if (stricmp(szKind, "commerce") == 0) return "commerce";
		if (stricmp(szKind, "yield") == 0) return "yield";
		return NULL;
	}

	int getInfoCountForKind(const char* szKind)
	{
		if (strcmp(szKind, "unit") == 0) return GC.getNumUnitInfos();
		if (strcmp(szKind, "unit_ai") == 0) return (int)GC.getUnitAIInfo().size();
		if (strcmp(szKind, "building") == 0) return GC.getNumBuildingInfos();
		if (strcmp(szKind, "building_class") == 0) return GC.getNumBuildingClassInfos();
		if (strcmp(szKind, "project") == 0) return GC.getNumProjectInfos();
		if (strcmp(szKind, "process") == 0) return GC.getNumProcessInfos();
		if (strcmp(szKind, "terrain") == 0) return GC.getNumTerrainInfos();
		if (strcmp(szKind, "feature") == 0) return GC.getNumFeatureInfos();
		if (strcmp(szKind, "bonus") == 0) return GC.getNumBonusInfos();
		if (strcmp(szKind, "improvement") == 0) return GC.getNumImprovementInfos();
		if (strcmp(szKind, "route") == 0) return GC.getNumRouteInfos();
		if (strcmp(szKind, "promotion") == 0) return GC.getNumPromotionInfos();
		if (strcmp(szKind, "tech") == 0) return GC.getNumTechInfos();
		if (strcmp(szKind, "civic") == 0) return GC.getNumCivicInfos();
		if (strcmp(szKind, "civic_option") == 0) return GC.getNumCivicOptionInfos();
		if (strcmp(szKind, "religion") == 0) return GC.getNumReligionInfos();
		if (strcmp(szKind, "corporation") == 0) return GC.getNumCorporationInfos();
		if (strcmp(szKind, "victory") == 0) return GC.getNumVictoryInfos();
		if (strcmp(szKind, "game_option") == 0) return GC.getNumGameOptionInfos();
		if (strcmp(szKind, "multiplayer_option") == 0) return GC.getNumMPOptionInfos();
		if (strcmp(szKind, "force_control") == 0) return GC.getNumForceControlInfos();
		if (strcmp(szKind, "era") == 0) return GC.getNumEraInfos();
		if (strcmp(szKind, "leader") == 0) return GC.getNumLeaderHeadInfos();
		if (strcmp(szKind, "civilization") == 0) return GC.getNumCivilizationInfos();
		if (strcmp(szKind, "handicap") == 0) return GC.getNumHandicapInfos();
		if (strcmp(szKind, "game_speed") == 0) return GC.getNumGameSpeedInfos();
		if (strcmp(szKind, "hurry") == 0) return GC.getNumHurryInfos();
		if (strcmp(szKind, "build") == 0) return GC.getNumBuildInfos();
		if (strcmp(szKind, "goody") == 0) return GC.getNumGoodyInfos();
		if (strcmp(szKind, "mission") == 0) return GC.getNumMissionInfos();
		if (strcmp(szKind, "espionage_mission") == 0) return GC.getNumEspionageMissionInfos();
		if (strcmp(szKind, "specialist") == 0) return GC.getNumSpecialistInfos();
		if (strcmp(szKind, "unit_class") == 0) return GC.getNumUnitClassInfos();
		if (strcmp(szKind, "unit_combat") == 0) return GC.getNumUnitCombatInfos();
		if (strcmp(szKind, "player_option") == 0) return GC.getNumPlayerOptionInfos();
		if (strcmp(szKind, "commerce") == 0) return GC.getNUM_COMMERCE_TYPES();
		if (strcmp(szKind, "yield") == 0) return GC.getNUM_YIELD_TYPES();
		return -1;
	}

	const CvInfoBase* getInfoBaseForKind(const char* szKind, int iId)
	{
		if (iId < 0 || iId >= getInfoCountForKind(szKind))
		{
			return NULL;
		}
		if (strcmp(szKind, "unit") == 0) return &GC.getUnitInfo((UnitTypes)iId);
		if (strcmp(szKind, "unit_ai") == 0) return &GC.getUnitAIInfo((UnitAITypes)iId);
		if (strcmp(szKind, "building") == 0) return &GC.getBuildingInfo((BuildingTypes)iId);
		if (strcmp(szKind, "building_class") == 0) return &GC.getBuildingClassInfo((BuildingClassTypes)iId);
		if (strcmp(szKind, "project") == 0) return &GC.getProjectInfo((ProjectTypes)iId);
		if (strcmp(szKind, "process") == 0) return &GC.getProcessInfo((ProcessTypes)iId);
		if (strcmp(szKind, "terrain") == 0) return &GC.getTerrainInfo((TerrainTypes)iId);
		if (strcmp(szKind, "feature") == 0) return &GC.getFeatureInfo((FeatureTypes)iId);
		if (strcmp(szKind, "bonus") == 0) return &GC.getBonusInfo((BonusTypes)iId);
		if (strcmp(szKind, "improvement") == 0) return &GC.getImprovementInfo((ImprovementTypes)iId);
		if (strcmp(szKind, "route") == 0) return &GC.getRouteInfo((RouteTypes)iId);
		if (strcmp(szKind, "promotion") == 0) return &GC.getPromotionInfo((PromotionTypes)iId);
		if (strcmp(szKind, "tech") == 0) return &GC.getTechInfo((TechTypes)iId);
		if (strcmp(szKind, "civic") == 0) return &GC.getCivicInfo((CivicTypes)iId);
		if (strcmp(szKind, "civic_option") == 0) return &GC.getCivicOptionInfo((CivicOptionTypes)iId);
		if (strcmp(szKind, "religion") == 0) return &GC.getReligionInfo((ReligionTypes)iId);
		if (strcmp(szKind, "corporation") == 0) return &GC.getCorporationInfo((CorporationTypes)iId);
		if (strcmp(szKind, "victory") == 0) return &GC.getVictoryInfo((VictoryTypes)iId);
		if (strcmp(szKind, "game_option") == 0) return &GC.getGameOptionInfo((GameOptionTypes)iId);
		if (strcmp(szKind, "multiplayer_option") == 0) return &GC.getMPOptionInfo((MultiplayerOptionTypes)iId);
		if (strcmp(szKind, "force_control") == 0) return &GC.getForceControlInfo((ForceControlTypes)iId);
		if (strcmp(szKind, "era") == 0) return &GC.getEraInfo((EraTypes)iId);
		if (strcmp(szKind, "leader") == 0) return &GC.getLeaderHeadInfo((LeaderHeadTypes)iId);
		if (strcmp(szKind, "civilization") == 0) return &GC.getCivilizationInfo((CivilizationTypes)iId);
		if (strcmp(szKind, "handicap") == 0) return &GC.getHandicapInfo((HandicapTypes)iId);
		if (strcmp(szKind, "game_speed") == 0) return &GC.getGameSpeedInfo((GameSpeedTypes)iId);
		if (strcmp(szKind, "hurry") == 0) return &GC.getHurryInfo((HurryTypes)iId);
		if (strcmp(szKind, "build") == 0) return &GC.getBuildInfo((BuildTypes)iId);
		if (strcmp(szKind, "goody") == 0) return &GC.getGoodyInfo((GoodyTypes)iId);
		if (strcmp(szKind, "mission") == 0) return &GC.getMissionInfo((MissionTypes)iId);
		if (strcmp(szKind, "espionage_mission") == 0) return &GC.getEspionageMissionInfo((EspionageMissionTypes)iId);
		if (strcmp(szKind, "specialist") == 0) return &GC.getSpecialistInfo((SpecialistTypes)iId);
		if (strcmp(szKind, "unit_class") == 0) return &GC.getUnitClassInfo((UnitClassTypes)iId);
		if (strcmp(szKind, "unit_combat") == 0) return &GC.getUnitCombatInfo((UnitCombatTypes)iId);
		if (strcmp(szKind, "player_option") == 0) return &GC.getPlayerOptionInfo((PlayerOptionTypes)iId);
		if (strcmp(szKind, "commerce") == 0) return &GC.getCommerceInfo((CommerceTypes)iId);
		if (strcmp(szKind, "yield") == 0) return &GC.getYieldInfo((YieldTypes)iId);
		return NULL;
	}

	bool validPlayer(int iPlayer)
	{
		return (iPlayer >= 0 && iPlayer < GC.getMAX_PLAYERS());
	}

	bool validTeam(int iTeam)
	{
		return (iTeam >= 0 && iTeam < MAX_TEAMS);
	}

	bool validEverTeam(int iTeam)
	{
		return validTeam(iTeam) && GET_TEAM((TeamTypes)iTeam).isEverAlive();
	}

	bool getTeamRelationArgs(JSON_Object* pArgs, int& iTeam, int& iOtherTeam)
	{
		if (!getInt(pArgs, "team", iTeam) || !validEverTeam(iTeam))
		{
			return false;
		}
		if (!getInt(pArgs, "other_team", iOtherTeam) || !validEverTeam(iOtherTeam))
		{
			return false;
		}
		return (iTeam != iOtherTeam);
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

	JSON_Value* makeResultReplyValue(int iId, JSON_Object** ppResult)
	{
		JSON_Value* pValue = makeBaseMessage("reply");
		JSON_Object* pObject = json_value_get_object(pValue);
		JSON_Value* pResultValue = json_value_init_object();
		JSON_Object* pResult = json_value_get_object(pResultValue);
		json_object_set_number(pObject, "id", iId);
		json_object_set_boolean(pObject, "ok", 1);
		json_object_set_value(pObject, "result", pResultValue);
		*ppResult = pResult;
		return pValue;
	}

	bool getPlotArgs(JSON_Object* pArgs, int& iX, int& iY, CvPlot*& pPlot)
	{
		if (!getInt(pArgs, "x", iX) || !getInt(pArgs, "y", iY))
		{
			return false;
		}
		pPlot = GC.getMapINLINE().plot(iX, iY);
		return (pPlot != NULL);
	}

	bool getCityArgs(JSON_Object* pArgs, int& iPlayer, int& iCity, CvCity*& pCity)
	{
		if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
		{
			return false;
		}
		if (!getInt(pArgs, "city", iCity))
		{
			return false;
		}
		pCity = GET_PLAYER((PlayerTypes)iPlayer).getCity(iCity);
		return (pCity != NULL);
	}

	bool getUnitArgs(JSON_Object* pArgs, int& iPlayer, int& iUnit, CvUnit*& pUnit)
	{
		if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
		{
			return false;
		}
		if (!getInt(pArgs, "unit", iUnit))
		{
			return false;
		}
		pUnit = GET_PLAYER((PlayerTypes)iPlayer).getUnit(iUnit);
		return (pUnit != NULL);
	}

	bool getPlayerArg(JSON_Object* pArgs, int& iPlayer)
	{
		return (getInt(pArgs, "player", iPlayer) && validPlayer(iPlayer));
	}

	void setPlayerState(JSON_Object* pResult, int iPlayer)
	{
		CvPlayer& kPlayer = GET_PLAYER((PlayerTypes)iPlayer);
		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_number(pResult, "team", kPlayer.getTeam());
		json_object_set_boolean(pResult, "alive", kPlayer.isAlive() ? 1 : 0);
		json_object_set_boolean(pResult, "ever_alive", kPlayer.isEverAlive() ? 1 : 0);
		json_object_set_boolean(pResult, "human", kPlayer.isHuman() ? 1 : 0);
		json_object_set_boolean(pResult, "barbarian", kPlayer.isBarbarian() ? 1 : 0);
		json_object_set_boolean(pResult, "minor", kPlayer.isMinorCiv() ? 1 : 0);
		json_object_set_boolean(pResult, "playable", kPlayer.isPlayable() ? 1 : 0);
		json_object_set_boolean(pResult, "founded_first_city", kPlayer.isFoundedFirstCity() ? 1 : 0);
		json_object_set_boolean(pResult, "extended_game", kPlayer.isExtendedGame() ? 1 : 0);
		json_object_set_boolean(pResult, "turn_active", kPlayer.isTurnActive() ? 1 : 0);
		json_object_set_boolean(pResult, "turn_done", kPlayer.isTurnDone() ? 1 : 0);
		json_object_set_boolean(pResult, "end_turn", kPlayer.isEndTurn() ? 1 : 0);
		json_object_set_boolean(pResult, "auto_moves", kPlayer.isAutoMoves() ? 1 : 0);
		json_object_set_boolean(pResult, "strike", kPlayer.isStrike() ? 1 : 0);
		json_object_set_number(pResult, "handicap", kPlayer.getHandicapType());
		json_object_set_number(pResult, "civilization", kPlayer.getCivilizationType());
		json_object_set_number(pResult, "leader", kPlayer.getLeaderType());
		json_object_set_number(pResult, "personality", kPlayer.getPersonalityType());
		json_object_set_number(pResult, "current_era", kPlayer.getCurrentEra());
		json_object_set_number(pResult, "parent", kPlayer.getParent());
		json_object_set_number(pResult, "player_color", kPlayer.getPlayerColor());
		json_object_set_number(pResult, "gold", kPlayer.getGold());
		json_object_set_number(pResult, "cities", kPlayer.getNumCities());
		json_object_set_number(pResult, "units", kPlayer.getNumUnits());
		json_object_set_number(pResult, "population", kPlayer.getTotalPopulation());
	}

	CvString makePlayerStateReply(int iId, int iPlayer)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		setPlayerState(pResult, iPlayer);
		return serializeAndFree(pValue);
	}

	void setPlayerEconomyState(JSON_Object* pResult, int iPlayer)
	{
		JSON_Value* pCommercePercentValue = json_value_init_array();
		JSON_Value* pCommerceRateValue = json_value_init_array();
		JSON_Value* pCommerceRateModifierValue = json_value_init_array();
		JSON_Array* pCommercePercent = json_value_get_array(pCommercePercentValue);
		JSON_Array* pCommerceRate = json_value_get_array(pCommerceRateValue);
		JSON_Array* pCommerceRateModifier = json_value_get_array(pCommerceRateModifierValue);
		CvPlayer& kPlayer = GET_PLAYER((PlayerTypes)iPlayer);
		int iI;

		for (iI = 0; iI < NUM_COMMERCE_TYPES; ++iI)
		{
			json_array_append_number(pCommercePercent, kPlayer.getCommercePercent((CommerceTypes)iI));
			json_array_append_number(pCommerceRate, kPlayer.getCommerceRate((CommerceTypes)iI));
			json_array_append_number(pCommerceRateModifier, kPlayer.getCommerceRateModifier((CommerceTypes)iI));
		}

		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_number(pResult, "gold", kPlayer.getGold());
		json_object_set_number(pResult, "gold_per_turn", kPlayer.getGoldPerTurn());
		json_object_set_number(pResult, "advanced_start_points", kPlayer.getAdvancedStartPoints());
		json_object_set_number(pResult, "golden_age_turns", kPlayer.getGoldenAgeTurns());
		json_object_set_number(pResult, "golden_age_length", kPlayer.getGoldenAgeLength());
		json_object_set_boolean(pResult, "golden_age", kPlayer.isGoldenAge() ? 1 : 0);
		json_object_set_number(pResult, "num_unit_golden_ages", kPlayer.getNumUnitGoldenAges());
		json_object_set_number(pResult, "units_required_for_golden_age", kPlayer.unitsRequiredForGoldenAge());
		json_object_set_number(pResult, "units_golden_age_ready", kPlayer.unitsGoldenAgeReady());
		json_object_set_number(pResult, "anarchy_turns", kPlayer.getAnarchyTurns());
		json_object_set_boolean(pResult, "anarchy", kPlayer.isAnarchy() ? 1 : 0);
		json_object_set_number(pResult, "strike_turns", kPlayer.getStrikeTurns());
		json_object_set_boolean(pResult, "strike", kPlayer.isStrike() ? 1 : 0);
		json_object_set_number(pResult, "combat_experience", kPlayer.getCombatExperience());
		json_object_set_number(pResult, "gold_per_unit", kPlayer.getGoldPerUnit());
		json_object_set_number(pResult, "gold_per_military_unit", kPlayer.getGoldPerMilitaryUnit());
		json_object_set_number(pResult, "total_culture", kPlayer.countTotalCulture());
		json_object_set_value(pResult, "commerce_percent", pCommercePercentValue);
		json_object_set_value(pResult, "commerce_rate", pCommerceRateValue);
		json_object_set_value(pResult, "commerce_rate_modifier", pCommerceRateModifierValue);
	}

	CvString makePlayerEconomyStateReply(int iId, int iPlayer)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		setPlayerEconomyState(pResult, iPlayer);
		return serializeAndFree(pValue);
	}

	CvString makePlayerGoldPerTurnStateReply(int iId, int iPlayer, int iOtherPlayer)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_number(pResult, "other_player", iOtherPlayer);
		json_object_set_number(pResult, "value", GET_PLAYER((PlayerTypes)iPlayer).getGoldPerTurnByPlayer((PlayerTypes)iOtherPlayer));
		return serializeAndFree(pValue);
	}

	void setCityState(JSON_Object* pResult, CvCity* pCity)
	{
		int iPlayer = pCity->getOwnerINLINE();
		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_number(pResult, "city", pCity->getID());
		json_object_set_number(pResult, "x", pCity->getX_INLINE());
		json_object_set_number(pResult, "y", pCity->getY_INLINE());
		json_object_set_number(pResult, "population", pCity->getPopulation());
		json_object_set_number(pResult, "culture", pCity->getCulture((PlayerTypes)iPlayer));
		json_object_set_number(pResult, "production", pCity->getProduction());
		json_object_set_number(pResult, "production_needed", pCity->getProductionNeeded());
		json_object_set_number(pResult, "production_unit", pCity->getProductionUnit());
		json_object_set_number(pResult, "production_unit_ai", pCity->getProductionUnitAI());
		json_object_set_number(pResult, "production_building", pCity->getProductionBuilding());
		json_object_set_number(pResult, "production_project", pCity->getProductionProject());
		json_object_set_number(pResult, "production_process", pCity->getProductionProcess());
		json_object_set_number(pResult, "order_queue_length", pCity->getOrderQueueLength());
		json_object_set_number(pResult, "occupation_timer", pCity->getOccupationTimer());
		json_object_set_number(pResult, "hurry_anger_timer", pCity->getHurryAngerTimer());
	}

	CvString makeCityStateReply(int iId, CvCity* pCity)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		setCityState(pResult, pCity);
		return serializeAndFree(pValue);
	}

	CvString makeCityDetailStateReply(int iId, CvCity* pCity)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		JSON_Value* pYieldValue = json_value_init_array();
		JSON_Array* pYields = json_value_get_array(pYieldValue);
		JSON_Value* pCommerceValue = json_value_init_array();
		JSON_Array* pCommerce = json_value_get_array(pCommerceValue);
		JSON_Value* pCommerceTimes100Value = json_value_init_array();
		JSON_Array* pCommerceTimes100 = json_value_get_array(pCommerceTimes100Value);

		for (int iYield = 0; iYield < GC.getNUM_YIELD_TYPES(); ++iYield)
		{
			json_array_append_number(pYields, pCity->getYieldRate((YieldTypes)iYield));
		}
		for (int iCommerce = 0; iCommerce < GC.getNUM_COMMERCE_TYPES(); ++iCommerce)
		{
			json_array_append_number(pCommerce, pCity->getCommerceRate((CommerceTypes)iCommerce));
			json_array_append_number(pCommerceTimes100, pCity->getCommerceRateTimes100((CommerceTypes)iCommerce));
		}

		json_object_set_number(pResult, "player", pCity->getOwnerINLINE());
		json_object_set_number(pResult, "city", pCity->getID());
		json_object_set_number(pResult, "x", pCity->getX_INLINE());
		json_object_set_number(pResult, "y", pCity->getY_INLINE());
		json_object_set_boolean(pResult, "production", pCity->isProduction() ? 1 : 0);
		json_object_set_boolean(pResult, "food_production", pCity->isFoodProduction() ? 1 : 0);
		json_object_set_boolean(pResult, "disorder", pCity->isDisorder() ? 1 : 0);
		json_object_set_boolean(pResult, "occupation", pCity->isOccupation() ? 1 : 0);
		json_object_set_boolean(pResult, "we_love_the_king_day", pCity->isWeLoveTheKingDay() ? 1 : 0);
		json_object_set_number(pResult, "food", pCity->getFood());
		json_object_set_number(pResult, "food_kept", pCity->getFoodKept());
		json_object_set_number(pResult, "growth_threshold", pCity->growthThreshold());
		json_object_set_number(pResult, "food_consumption", pCity->foodConsumption(false, 0));
		json_object_set_number(pResult, "food_difference", pCity->foodDifference(true));
		json_object_set_number(pResult, "happy_level", pCity->happyLevel());
		json_object_set_number(pResult, "unhappy_level", pCity->unhappyLevel(0));
		json_object_set_number(pResult, "angry_population", pCity->angryPopulation(0));
		json_object_set_number(pResult, "good_health", pCity->goodHealth());
		json_object_set_number(pResult, "bad_health", pCity->badHealth(false, 0));
		json_object_set_number(pResult, "health_rate", pCity->healthRate(false, 0));
		json_object_set_number(pResult, "unhealthy_population", pCity->unhealthyPopulation(false, 0));
		json_object_set_number(pResult, "maintenance", pCity->getMaintenance());
		json_object_set_number(pResult, "distance_maintenance", pCity->calculateDistanceMaintenance());
		json_object_set_number(pResult, "num_cities_maintenance", pCity->calculateNumCitiesMaintenance());
		json_object_set_number(pResult, "colony_maintenance", pCity->calculateColonyMaintenance());
		json_object_set_number(pResult, "corporation_maintenance", pCity->calculateCorporationMaintenance());
		json_object_set_number(pResult, "production_left", pCity->productionLeft());
		json_object_set_number(pResult, "current_production_difference", pCity->getCurrentProductionDifference(false, true));
		json_object_set_number(pResult, "defense_damage", pCity->getDefenseDamage());
		json_object_set_number(pResult, "total_defense", pCity->getTotalDefense(false));
		json_object_set_number(pResult, "defense_modifier", pCity->getDefenseModifier(false));
		json_object_set_value(pResult, "yield_rate", pYieldValue);
		json_object_set_value(pResult, "commerce_rate", pCommerceValue);
		json_object_set_value(pResult, "commerce_rate_times100", pCommerceTimes100Value);
		return serializeAndFree(pValue);
	}

	CvString makeCityProductionOptionsReply(int iId, CvCity* pCity, int iContinueCurrent, int iTestVisible, int iIgnoreCost, int iIgnoreUpgrades)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		JSON_Value* pUnitsValue = json_value_init_array();
		JSON_Array* pUnits = json_value_get_array(pUnitsValue);
		JSON_Value* pBuildingsValue = json_value_init_array();
		JSON_Array* pBuildings = json_value_get_array(pBuildingsValue);
		JSON_Value* pProjectsValue = json_value_init_array();
		JSON_Array* pProjects = json_value_get_array(pProjectsValue);
		JSON_Value* pProcessesValue = json_value_init_array();
		JSON_Array* pProcesses = json_value_get_array(pProcessesValue);

		bool bContinueCurrent = (iContinueCurrent != 0);
		bool bTestVisible = (iTestVisible != 0);
		bool bIgnoreCost = (iIgnoreCost != 0);
		bool bIgnoreUpgrades = (iIgnoreUpgrades != 0);

		for (int iUnit = 0; iUnit < GC.getNumUnitInfos(); ++iUnit)
		{
			if (pCity->canTrain((UnitTypes)iUnit, bContinueCurrent, bTestVisible, bIgnoreCost, bIgnoreUpgrades))
			{
				json_array_append_number(pUnits, iUnit);
			}
		}
		for (int iBuilding = 0; iBuilding < GC.getNumBuildingInfos(); ++iBuilding)
		{
			if (pCity->canConstruct((BuildingTypes)iBuilding, bContinueCurrent, bTestVisible, bIgnoreCost))
			{
				json_array_append_number(pBuildings, iBuilding);
			}
		}
		for (int iProject = 0; iProject < GC.getNumProjectInfos(); ++iProject)
		{
			if (pCity->canCreate((ProjectTypes)iProject, bContinueCurrent, bTestVisible))
			{
				json_array_append_number(pProjects, iProject);
			}
		}
		for (int iProcess = 0; iProcess < GC.getNumProcessInfos(); ++iProcess)
		{
			if (pCity->canMaintain((ProcessTypes)iProcess, bContinueCurrent))
			{
				json_array_append_number(pProcesses, iProcess);
			}
		}

		json_object_set_number(pResult, "player", pCity->getOwnerINLINE());
		json_object_set_number(pResult, "city", pCity->getID());
		json_object_set_boolean(pResult, "continue_current", bContinueCurrent ? 1 : 0);
		json_object_set_boolean(pResult, "test_visible", bTestVisible ? 1 : 0);
		json_object_set_boolean(pResult, "ignore_cost", bIgnoreCost ? 1 : 0);
		json_object_set_boolean(pResult, "ignore_upgrades", bIgnoreUpgrades ? 1 : 0);
		json_object_set_value(pResult, "units", pUnitsValue);
		json_object_set_value(pResult, "buildings", pBuildingsValue);
		json_object_set_value(pResult, "projects", pProjectsValue);
		json_object_set_value(pResult, "processes", pProcessesValue);
		return serializeAndFree(pValue);
	}

	CvString makeCityBuildingStateReply(int iId, CvCity* pCity, int iBuilding)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		int iReal = pCity->getNumRealBuilding((BuildingTypes)iBuilding);
		int iFree = pCity->getNumFreeBuilding((BuildingTypes)iBuilding);
		json_object_set_number(pResult, "player", pCity->getOwnerINLINE());
		json_object_set_number(pResult, "city", pCity->getID());
		json_object_set_number(pResult, "building", iBuilding);
		json_object_set_number(pResult, "real", iReal);
		json_object_set_number(pResult, "free", iFree);
		json_object_set_boolean(pResult, "active", pCity->getNumBuilding((BuildingTypes)iBuilding) > 0 ? 1 : 0);
		return serializeAndFree(pValue);
	}

	CvString makeCityReligionStateReply(int iId, CvCity* pCity, int iReligion)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "player", pCity->getOwnerINLINE());
		json_object_set_number(pResult, "city", pCity->getID());
		json_object_set_number(pResult, "religion", iReligion);
		json_object_set_boolean(pResult, "has", pCity->isHasReligion((ReligionTypes)iReligion) ? 1 : 0);
		return serializeAndFree(pValue);
	}

	CvString makeCityCorporationStateReply(int iId, CvCity* pCity, int iCorporation)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "player", pCity->getOwnerINLINE());
		json_object_set_number(pResult, "city", pCity->getID());
		json_object_set_number(pResult, "corporation", iCorporation);
		json_object_set_boolean(pResult, "has", pCity->isHasCorporation((CorporationTypes)iCorporation) ? 1 : 0);
		return serializeAndFree(pValue);
	}

	CvString makeCityBuildingClassChangeReply(int iId, CvCity* pCity, int iBuildingClass)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "player", pCity->getOwnerINLINE());
		json_object_set_number(pResult, "city", pCity->getID());
		json_object_set_number(pResult, "building_class", iBuildingClass);
		json_object_set_number(pResult, "happiness", pCity->getBuildingHappyChange((BuildingClassTypes)iBuildingClass));
		json_object_set_number(pResult, "health", pCity->getBuildingHealthChange((BuildingClassTypes)iBuildingClass));
		return serializeAndFree(pValue);
	}

	void setUnitState(JSON_Object* pResult, CvUnit* pUnit)
	{
		JSON_Value* pPromotionsValue = json_value_init_array();
		JSON_Array* pPromotions = json_value_get_array(pPromotionsValue);

		json_object_set_number(pResult, "player", pUnit->getOwnerINLINE());
		json_object_set_number(pResult, "unit", pUnit->getID());
		json_object_set_number(pResult, "unit_type", pUnit->getUnitType());
		json_object_set_number(pResult, "unit_ai", pUnit->AI_getUnitAIType());
		json_object_set_number(pResult, "domain", pUnit->getDomainType());
		json_object_set_number(pResult, "x", pUnit->getX_INLINE());
		json_object_set_number(pResult, "y", pUnit->getY_INLINE());
		json_object_set_number(pResult, "damage", pUnit->getDamage());
		json_object_set_number(pResult, "experience", pUnit->getExperience());
		json_object_set_number(pResult, "level", pUnit->getLevel());
		json_object_set_number(pResult, "moves", pUnit->getMoves());
		json_object_set_number(pResult, "max_moves", pUnit->maxMoves());
		json_object_set_number(pResult, "base_combat", pUnit->baseCombatStr());
		json_object_set_number(pResult, "cargo", pUnit->getCargo());
		json_object_set_number(pResult, "fortify_turns", pUnit->getFortifyTurns());
		json_object_set_number(pResult, "immobile_timer", pUnit->getImmobileTimer());
		json_object_set_boolean(pResult, "made_attack", pUnit->isMadeAttack() ? 1 : 0);

		for (int iPromotion = 0; iPromotion < GC.getNumPromotionInfos(); ++iPromotion)
		{
			if (pUnit->isHasPromotion((PromotionTypes)iPromotion))
			{
				json_array_append_number(pPromotions, iPromotion);
			}
		}

		json_object_set_value(pResult, "promotions", pPromotionsValue);
	}

	CvString makeUnitStateReply(int iId, CvUnit* pUnit)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		setUnitState(pResult, pUnit);
		return serializeAndFree(pValue);
	}

	CvString makeUnitDetailStateReply(int iId, CvUnit* pUnit)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		CvPlot* pPlot = pUnit->plot();
		json_object_set_number(pResult, "player", pUnit->getOwnerINLINE());
		json_object_set_number(pResult, "unit", pUnit->getID());
		json_object_set_number(pResult, "unit_type", pUnit->getUnitType());
		json_object_set_number(pResult, "unit_ai", pUnit->AI_getUnitAIType());
		json_object_set_number(pResult, "domain", pUnit->getDomainType());
		json_object_set_number(pResult, "unit_combat", pUnit->getUnitCombatType());
		json_object_set_number(pResult, "special_unit", pUnit->getSpecialUnitType());
		json_object_set_number(pResult, "x", pUnit->getX_INLINE());
		json_object_set_number(pResult, "y", pUnit->getY_INLINE());
		json_object_set_number(pResult, "area", pUnit->getArea());
		json_object_set_number(pResult, "group", pUnit->getGroupID());
		json_object_set_boolean(pResult, "in_group", pUnit->isInGroup() ? 1 : 0);
		json_object_set_boolean(pResult, "group_head", pUnit->isGroupHead() ? 1 : 0);
		json_object_set_number(pResult, "base_moves", pUnit->baseMoves());
		json_object_set_number(pResult, "max_moves", pUnit->maxMoves());
		json_object_set_number(pResult, "moves_left", pUnit->movesLeft());
		json_object_set_boolean(pResult, "can_move", pUnit->canMove() ? 1 : 0);
		json_object_set_boolean(pResult, "has_moved", pUnit->hasMoved() ? 1 : 0);
		json_object_set_number(pResult, "visibility_range", pUnit->visibilityRange());
		json_object_set_number(pResult, "air_range", pUnit->airRange());
		json_object_set_number(pResult, "nuke_range", pUnit->nukeRange());
		json_object_set_boolean(pResult, "can_build_route", pUnit->canBuildRoute() ? 1 : 0);
		json_object_set_number(pResult, "build_type", pUnit->getBuildType());
		json_object_set_number(pResult, "work_rate", pUnit->workRate(false));
		json_object_set_number(pResult, "max_work_rate", pUnit->workRate(true));
		json_object_set_boolean(pResult, "can_fight", pUnit->canFight() ? 1 : 0);
		json_object_set_boolean(pResult, "can_attack", pUnit->canAttack() ? 1 : 0);
		json_object_set_boolean(pResult, "can_defend", pUnit->canDefend(pPlot) ? 1 : 0);
		json_object_set_boolean(pResult, "fighting", pUnit->isFighting() ? 1 : 0);
		json_object_set_boolean(pResult, "attacking", pUnit->isAttacking() ? 1 : 0);
		json_object_set_boolean(pResult, "defending", pUnit->isDefending() ? 1 : 0);
		json_object_set_boolean(pResult, "combat", pUnit->isCombat() ? 1 : 0);
		json_object_set_boolean(pResult, "hurt", pUnit->isHurt() ? 1 : 0);
		json_object_set_boolean(pResult, "dead", pUnit->isDead() ? 1 : 0);
		json_object_set_number(pResult, "max_hit_points", pUnit->maxHitPoints());
		json_object_set_number(pResult, "curr_hit_points", pUnit->currHitPoints());
		json_object_set_number(pResult, "base_combat", pUnit->baseCombatStr());
		json_object_set_number(pResult, "curr_combat", pUnit->currCombatStr(pPlot, NULL));
		json_object_set_number(pResult, "combat_limit", pUnit->combatLimit());
		json_object_set_number(pResult, "air_combat_limit", pUnit->airCombatLimit());
		json_object_set_number(pResult, "fortify_modifier", pUnit->fortifyModifier());
		json_object_set_number(pResult, "experience_needed", pUnit->experienceNeeded());
		json_object_set_number(pResult, "attack_xp_value", pUnit->attackXPValue());
		json_object_set_number(pResult, "defense_xp_value", pUnit->defenseXPValue());
		json_object_set_number(pResult, "max_xp_value", pUnit->maxXPValue());
		json_object_set_number(pResult, "special_cargo", pUnit->specialCargo());
		json_object_set_number(pResult, "domain_cargo", pUnit->domainCargo());
		json_object_set_number(pResult, "cargo", pUnit->getCargo());
		json_object_set_number(pResult, "cargo_space", pUnit->cargoSpace());
		json_object_set_number(pResult, "cargo_space_available", pUnit->cargoSpaceAvailable());
		json_object_set_boolean(pResult, "has_cargo", pUnit->hasCargo() ? 1 : 0);
		json_object_set_boolean(pResult, "full", pUnit->isFull() ? 1 : 0);
		json_object_set_boolean(pResult, "cargo_can_move", pUnit->canCargoAllMove() ? 1 : 0);
		json_object_set_boolean(pResult, "automated", pUnit->isAutomated() ? 1 : 0);
		json_object_set_boolean(pResult, "waiting", pUnit->isWaiting() ? 1 : 0);
		json_object_set_boolean(pResult, "fortifyable", pUnit->isFortifyable() ? 1 : 0);
		json_object_set_boolean(pResult, "made_interception", pUnit->isMadeInterception() ? 1 : 0);
		json_object_set_boolean(pResult, "promotion_ready", pUnit->isPromotionReady() ? 1 : 0);
		json_object_set_boolean(pResult, "animal", pUnit->isAnimal() ? 1 : 0);
		json_object_set_boolean(pResult, "only_defensive", pUnit->isOnlyDefensive() ? 1 : 0);
		json_object_set_boolean(pResult, "rival_territory", pUnit->isRivalTerritory() ? 1 : 0);
		json_object_set_boolean(pResult, "military_happiness", pUnit->isMilitaryHappiness() ? 1 : 0);
		json_object_set_boolean(pResult, "spy", pUnit->isSpy() ? 1 : 0);
		json_object_set_boolean(pResult, "found", pUnit->isFound() ? 1 : 0);
		json_object_set_boolean(pResult, "golden_age", pUnit->isGoldenAge() ? 1 : 0);
		json_object_set_number(pResult, "last_move_turn", pUnit->getLastMoveTurn());
		json_object_set_number(pResult, "game_turn_created", pUnit->getGameTurnCreated());
		json_object_set_number(pResult, "experience_percent", pUnit->getExperiencePercent());
		return serializeAndFree(pValue);
	}

	CvString makeUnitPromotionStateReply(int iId, CvUnit* pUnit, int iPromotion)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "player", pUnit->getOwnerINLINE());
		json_object_set_number(pResult, "unit", pUnit->getID());
		json_object_set_number(pResult, "promotion", iPromotion);
		json_object_set_boolean(pResult, "has", pUnit->isHasPromotion((PromotionTypes)iPromotion) ? 1 : 0);
		return serializeAndFree(pValue);
	}

	CvString makePlotStateReply(int iId, CvPlot* pPlot)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		CvCity* pCity = pPlot->getPlotCity();
		json_object_set_number(pResult, "x", pPlot->getX_INLINE());
		json_object_set_number(pResult, "y", pPlot->getY_INLINE());
		json_object_set_number(pResult, "owner", pPlot->getOwnerINLINE());
		json_object_set_number(pResult, "terrain", pPlot->getTerrainType());
		json_object_set_number(pResult, "feature", pPlot->getFeatureType());
		json_object_set_number(pResult, "bonus", pPlot->getBonusType(NO_TEAM));
		json_object_set_number(pResult, "improvement", pPlot->getImprovementType());
		json_object_set_number(pResult, "route", pPlot->getRouteType());
		json_object_set_boolean(pResult, "water", pPlot->isWater() ? 1 : 0);
		json_object_set_boolean(pResult, "peak", pPlot->isPeak() ? 1 : 0);
		json_object_set_number(pResult, "units", pPlot->getNumUnits());
		json_object_set_number(pResult, "city_player", (pCity != NULL) ? pCity->getOwnerINLINE() : -1);
		json_object_set_number(pResult, "city", (pCity != NULL) ? pCity->getID() : -1);
		return serializeAndFree(pValue);
	}

	CvString makePlotCultureStateReply(int iId, CvPlot* pPlot, int iPlayer)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "x", pPlot->getX_INLINE());
		json_object_set_number(pResult, "y", pPlot->getY_INLINE());
		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_number(pResult, "culture", pPlot->getCulture((PlayerTypes)iPlayer));
		json_object_set_number(pResult, "total_culture", pPlot->countTotalCulture());
		json_object_set_number(pResult, "culture_percent", pPlot->calculateCulturePercent((PlayerTypes)iPlayer));
		return serializeAndFree(pValue);
	}

	CvString makePlotVisibilityStateReply(int iId, CvPlot* pPlot, int iTeam, int iDebug)
	{
		bool bDebug = (iDebug != 0);
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "x", pPlot->getX_INLINE());
		json_object_set_number(pResult, "y", pPlot->getY_INLINE());
		json_object_set_number(pResult, "team", iTeam);
		json_object_set_boolean(pResult, "debug", bDebug ? 1 : 0);
		json_object_set_boolean(pResult, "visible", pPlot->isVisible((TeamTypes)iTeam, bDebug) ? 1 : 0);
		json_object_set_boolean(pResult, "revealed", pPlot->isRevealed((TeamTypes)iTeam, bDebug) ? 1 : 0);
		json_object_set_number(pResult, "revealed_owner", pPlot->getRevealedOwner((TeamTypes)iTeam, bDebug));
		json_object_set_number(pResult, "revealed_team", pPlot->getRevealedTeam((TeamTypes)iTeam, bDebug));
		json_object_set_number(pResult, "revealed_improvement", pPlot->getRevealedImprovementType((TeamTypes)iTeam, bDebug));
		json_object_set_number(pResult, "revealed_route", pPlot->getRevealedRouteType((TeamTypes)iTeam, bDebug));
		return serializeAndFree(pValue);
	}

	CvString makePlayersListReply(int iId)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		JSON_Value* pPlayersValue = json_value_init_array();
		JSON_Array* pPlayers = json_value_get_array(pPlayersValue);

		for (int iPlayer = 0; iPlayer < GC.getMAX_PLAYERS(); ++iPlayer)
		{
			CvPlayer& kPlayer = GET_PLAYER((PlayerTypes)iPlayer);
			if (!kPlayer.isEverAlive())
			{
				continue;
			}

			JSON_Value* pPlayerValue = json_value_init_object();
			JSON_Object* pPlayer = json_value_get_object(pPlayerValue);
			setPlayerState(pPlayer, iPlayer);
			json_array_append_value(pPlayers, pPlayerValue);
		}

		json_object_set_value(pResult, "players", pPlayersValue);
		return serializeAndFree(pValue);
	}

	void setPlayerOptionsState(JSON_Object* pResult, int iPlayer)
	{
		CvPlayer& kPlayer = GET_PLAYER((PlayerTypes)iPlayer);
		JSON_Value* pCivicsValue = json_value_init_array();
		JSON_Array* pCivics = json_value_get_array(pCivicsValue);

		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_number(pResult, "team", kPlayer.getTeam());
		json_object_set_number(pResult, "state_religion", kPlayer.getStateReligion());
		json_object_set_number(pResult, "current_research", kPlayer.getCurrentResearch());

		for (int iOption = 0; iOption < GC.getNumCivicOptionInfos(); ++iOption)
		{
			json_array_append_number(pCivics, kPlayer.getCivics((CivicOptionTypes)iOption));
		}

		json_object_set_value(pResult, "civics", pCivicsValue);
	}

	CvString makePlayerOptionsReply(int iId, int iPlayer)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		setPlayerOptionsState(pResult, iPlayer);
		return serializeAndFree(pValue);
	}

	CvString makeTeamTechStateReply(int iId, int iTeam, int iTech)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		CvTeam& kTeam = GET_TEAM((TeamTypes)iTeam);
		json_object_set_number(pResult, "team", iTeam);
		json_object_set_number(pResult, "tech", iTech);
		json_object_set_boolean(pResult, "has", kTeam.isHasTech((TechTypes)iTech) ? 1 : 0);
		json_object_set_number(pResult, "progress", kTeam.getResearchProgress((TechTypes)iTech));
		return serializeAndFree(pValue);
	}

	void setBridgeGameState(JSON_Object* pResult)
	{
		CvGame& kGame = GC.getGameINLINE();
		json_object_set_number(pResult, "turn", kGame.getGameTurn());
		json_object_set_number(pResult, "year", kGame.getGameTurnYear());
		json_object_set_number(pResult, "elapsed_turns", kGame.getElapsedGameTurns());
		json_object_set_number(pResult, "start_turn", kGame.getStartTurn());
		json_object_set_number(pResult, "start_year", kGame.getStartYear());
		json_object_set_number(pResult, "estimate_end_turn", kGame.getEstimateEndTurn());
		json_object_set_number(pResult, "max_turns", kGame.getMaxTurns());
		json_object_set_number(pResult, "max_city_elimination", kGame.getMaxCityElimination());
		json_object_set_number(pResult, "advanced_start_points", kGame.getNumAdvancedStartPoints());
		json_object_set_number(pResult, "target_score", kGame.getTargetScore());
		json_object_set_number(pResult, "active_player", kGame.getActivePlayer());
		json_object_set_number(pResult, "active_team", kGame.getActiveTeam());
		json_object_set_number(pResult, "pause_player", kGame.getPausePlayer());
		json_object_set_boolean(pResult, "paused", kGame.isPaused() ? 1 : 0);
		json_object_set_number(pResult, "winner", kGame.getWinner());
		json_object_set_number(pResult, "victory", kGame.getVictory());
		json_object_set_number(pResult, "game_state", kGame.getGameState());
		json_object_set_number(pResult, "start_era", kGame.getStartEra());
		json_object_set_number(pResult, "current_era", kGame.getCurrentEra());
		json_object_set_number(pResult, "calendar", kGame.getCalendar());
		json_object_set_number(pResult, "game_speed", kGame.getGameSpeedType());
		json_object_set_number(pResult, "handicap", kGame.getHandicapType());
		json_object_set_number(pResult, "num_cities", kGame.getNumCities());
		json_object_set_number(pResult, "num_civ_cities", kGame.getNumCivCities());
		json_object_set_number(pResult, "total_population", kGame.getTotalPopulation());
		json_object_set_number(pResult, "num_human_players", kGame.getNumHumanPlayers());
		json_object_set_number(pResult, "num_deals", kGame.getNumDeals());
		json_object_set_number(pResult, "nukes_exploded", kGame.getNukesExploded());
		json_object_set_number(pResult, "ai_auto_play", kGame.getAIAutoPlay());
		json_object_set_boolean(pResult, "network_multiplayer", kGame.isNetworkMultiPlayer() ? 1 : 0);
		json_object_set_boolean(pResult, "game_multiplayer", kGame.isGameMultiPlayer() ? 1 : 0);
		json_object_set_boolean(pResult, "team_game", kGame.isTeamGame() ? 1 : 0);
		json_object_set_boolean(pResult, "debug_mode", kGame.isDebugMode() ? 1 : 0);
		json_object_set_boolean(pResult, "final_initialized", kGame.isFinalInitialized() ? 1 : 0);
	}

	CvString makeGameStateReply(int iId)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		setBridgeGameState(pResult);
		return serializeAndFree(pValue);
	}

	CvString makeGameOptionStateReply(int iId, int iOption)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "option", iOption);
		json_object_set_boolean(pResult, "enabled", GC.getGameINLINE().isOption((GameOptionTypes)iOption) ? 1 : 0);
		return serializeAndFree(pValue);
	}

	CvString makeMultiplayerOptionStateReply(int iId, int iOption)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "option", iOption);
		json_object_set_boolean(pResult, "enabled", GC.getGameINLINE().isMPOption((MultiplayerOptionTypes)iOption) ? 1 : 0);
		return serializeAndFree(pValue);
	}

	CvString makeForceControlStateReply(int iId, int iControl)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "control", iControl);
		json_object_set_boolean(pResult, "enabled", GC.getGameINLINE().isForcedControl((ForceControlTypes)iControl) ? 1 : 0);
		return serializeAndFree(pValue);
	}

	void setTeamState(JSON_Object* pResult, int iTeam)
	{
		CvTeam& kTeam = GET_TEAM((TeamTypes)iTeam);
		json_object_set_number(pResult, "team", iTeam);
		json_object_set_boolean(pResult, "alive", kTeam.isAlive() ? 1 : 0);
		json_object_set_boolean(pResult, "ever_alive", kTeam.isEverAlive() ? 1 : 0);
		json_object_set_boolean(pResult, "human", kTeam.isHuman() ? 1 : 0);
		json_object_set_boolean(pResult, "barbarian", kTeam.isBarbarian() ? 1 : 0);
		json_object_set_boolean(pResult, "minor", kTeam.isMinorCiv() ? 1 : 0);
		json_object_set_number(pResult, "leader", kTeam.getLeaderID());
		json_object_set_number(pResult, "secretary", kTeam.getSecretaryID());
		json_object_set_number(pResult, "members", kTeam.getNumMembers());
		json_object_set_number(pResult, "cities", kTeam.getNumCities());
		json_object_set_number(pResult, "population", kTeam.getTotalPopulation());
		json_object_set_number(pResult, "land", kTeam.getTotalLand());
		json_object_set_number(pResult, "assets", kTeam.getAssets());
		json_object_set_number(pResult, "power", kTeam.getPower(true));
		json_object_set_number(pResult, "defensive_power", kTeam.getDefensivePower());
		json_object_set_number(pResult, "at_war_count", kTeam.getAtWarCount(false));
		json_object_set_number(pResult, "has_met_count", kTeam.getHasMetCivCount(false));
		json_object_set_number(pResult, "defensive_pact_count", kTeam.getDefensivePactCount());
		json_object_set_number(pResult, "vassal_count", kTeam.getVassalCount());
		json_object_set_boolean(pResult, "vassal", kTeam.isAVassal() ? 1 : 0);
		json_object_set_number(pResult, "nuke_interception", kTeam.getNukeInterception());
		json_object_set_boolean(pResult, "map_trading", kTeam.isMapTrading() ? 1 : 0);
		json_object_set_boolean(pResult, "tech_trading", kTeam.isTechTrading() ? 1 : 0);
		json_object_set_boolean(pResult, "gold_trading", kTeam.isGoldTrading() ? 1 : 0);
		json_object_set_boolean(pResult, "open_borders_trading", kTeam.isOpenBordersTrading() ? 1 : 0);
		json_object_set_boolean(pResult, "defensive_pact_trading", kTeam.isDefensivePactTrading() ? 1 : 0);
		json_object_set_boolean(pResult, "permanent_alliance_trading", kTeam.isPermanentAllianceTrading() ? 1 : 0);
		json_object_set_boolean(pResult, "vassal_trading", kTeam.isVassalStateTrading() ? 1 : 0);
	}

	CvString makeTeamStateReply(int iId, int iTeam)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		setTeamState(pResult, iTeam);
		return serializeAndFree(pValue);
	}

	CvString makeTeamsListReply(int iId)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		JSON_Value* pTeamsValue = json_value_init_array();
		JSON_Array* pTeams = json_value_get_array(pTeamsValue);

		for (int iTeam = 0; iTeam < MAX_TEAMS; ++iTeam)
		{
			CvTeam& kTeam = GET_TEAM((TeamTypes)iTeam);
			if (!kTeam.isEverAlive())
			{
				continue;
			}

			JSON_Value* pTeamValue = json_value_init_object();
			JSON_Object* pTeamObject = json_value_get_object(pTeamValue);
			setTeamState(pTeamObject, iTeam);
			json_array_append_value(pTeams, pTeamValue);
		}

		json_object_set_value(pResult, "teams", pTeamsValue);
		return serializeAndFree(pValue);
	}

	CvString makeTeamRelationStateReply(int iId, int iTeam, int iOtherTeam)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		CvTeam& kTeam = GET_TEAM((TeamTypes)iTeam);
		json_object_set_number(pResult, "team", iTeam);
		json_object_set_number(pResult, "other_team", iOtherTeam);
		json_object_set_boolean(pResult, "has_met", kTeam.isHasMet((TeamTypes)iOtherTeam) ? 1 : 0);
		json_object_set_boolean(pResult, "at_war", kTeam.isAtWar((TeamTypes)iOtherTeam) ? 1 : 0);
		json_object_set_boolean(pResult, "can_declare_war", kTeam.canDeclareWar((TeamTypes)iOtherTeam) ? 1 : 0);
		json_object_set_boolean(pResult, "can_change_war_peace", kTeam.canChangeWarPeace((TeamTypes)iOtherTeam, true) ? 1 : 0);
		json_object_set_boolean(pResult, "permanent_war_peace", kTeam.isPermanentWarPeace((TeamTypes)iOtherTeam) ? 1 : 0);
		json_object_set_boolean(pResult, "open_borders", kTeam.isOpenBorders((TeamTypes)iOtherTeam) ? 1 : 0);
		json_object_set_boolean(pResult, "defensive_pact", kTeam.isDefensivePact((TeamTypes)iOtherTeam) ? 1 : 0);
		json_object_set_boolean(pResult, "force_peace", kTeam.isForcePeace((TeamTypes)iOtherTeam) ? 1 : 0);
		json_object_set_boolean(pResult, "vassal", kTeam.isVassal((TeamTypes)iOtherTeam) ? 1 : 0);
		json_object_set_boolean(pResult, "master", GET_TEAM((TeamTypes)iOtherTeam).isVassal((TeamTypes)iTeam) ? 1 : 0);
		json_object_set_number(pResult, "war_weariness", kTeam.getWarWeariness((TeamTypes)iOtherTeam));
		json_object_set_number(pResult, "stolen_visibility_timer", kTeam.getStolenVisibilityTimer((TeamTypes)iOtherTeam));
		json_object_set_number(pResult, "war_plan", kTeam.AI_getWarPlan((TeamTypes)iOtherTeam));
		return serializeAndFree(pValue);
	}

	CvString makeCitiesListReply(int iId, int iPlayer)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		JSON_Value* pCitiesValue = json_value_init_array();
		JSON_Array* pCities = json_value_get_array(pCitiesValue);
		CvPlayer& kPlayer = GET_PLAYER((PlayerTypes)iPlayer);
		int iLoop = 0;

		for (CvCity* pCity = kPlayer.firstCity(&iLoop); pCity != NULL; pCity = kPlayer.nextCity(&iLoop))
		{
			JSON_Value* pCityValue = json_value_init_object();
			JSON_Object* pCityObject = json_value_get_object(pCityValue);
			setCityState(pCityObject, pCity);
			json_array_append_value(pCities, pCityValue);
		}

		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_value(pResult, "cities", pCitiesValue);
		return serializeAndFree(pValue);
	}

	CvString makeUnitsListReply(int iId, int iPlayer)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		JSON_Value* pUnitsValue = json_value_init_array();
		JSON_Array* pUnits = json_value_get_array(pUnitsValue);
		CvPlayer& kPlayer = GET_PLAYER((PlayerTypes)iPlayer);
		int iLoop = 0;

		for (CvUnit* pUnit = kPlayer.firstUnit(&iLoop); pUnit != NULL; pUnit = kPlayer.nextUnit(&iLoop))
		{
			JSON_Value* pUnitValue = json_value_init_object();
			JSON_Object* pUnitObject = json_value_get_object(pUnitValue);
			setUnitState(pUnitObject, pUnit);
			json_array_append_value(pUnits, pUnitValue);
		}

		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_value(pResult, "units", pUnitsValue);
		return serializeAndFree(pValue);
	}

	CvString makeInfoCountReply(int iId, const char* szKind)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_string(pResult, "kind", szKind);
		json_object_set_number(pResult, "count", getInfoCountForKind(szKind));
		return serializeAndFree(pValue);
	}

	CvString makeInfoTypeReply(int iId, const char* szKind, int iInfo)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		const CvInfoBase* pInfo = getInfoBaseForKind(szKind, iInfo);
		json_object_set_string(pResult, "kind", szKind);
		json_object_set_number(pResult, "id", iInfo);
		json_object_set_string(pResult, "type", pInfo != NULL && pInfo->getType() != NULL ? pInfo->getType() : "");
		return serializeAndFree(pValue);
	}

	CvString makeInfoTypesListReply(int iId, const char* szKind)
	{
		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		JSON_Value* pTypesValue = json_value_init_array();
		JSON_Array* pTypes = json_value_get_array(pTypesValue);
		int iCount = getInfoCountForKind(szKind);

		for (int iI = 0; iI < iCount; ++iI)
		{
			const CvInfoBase* pInfo = getInfoBaseForKind(szKind, iI);
			JSON_Value* pTypeValue = json_value_init_object();
			JSON_Object* pType = json_value_get_object(pTypeValue);
			json_object_set_number(pType, "id", iI);
			json_object_set_string(pType, "type", pInfo != NULL && pInfo->getType() != NULL ? pInfo->getType() : "");
			json_array_append_value(pTypes, pTypeValue);
		}

		json_object_set_string(pResult, "kind", szKind);
		json_object_set_value(pResult, "types", pTypesValue);
		return serializeAndFree(pValue);
	}

	CvString handleQuery(int iId, const char* szName, JSON_Object* pArgs)
	{
		if (strcmp(szName, "get_game_turn") == 0)
		{
			CvString szReply;
			setReplyResultInt(szReply, iId, "turn", GC.getGameINLINE().getGameTurn());
			return szReply;
		}

		if (strcmp(szName, "get_game_state") == 0)
		{
			return makeGameStateReply(iId);
		}

		if (strcmp(szName, "get_info_count") == 0)
		{
			const char* szKind = getCanonicalInfoKind(json_object_get_string(pArgs, "kind"));
			if (szKind == NULL)
			{
				return makeErrorReply(iId, "bad_kind", "kind is missing or unsupported");
			}
			return makeInfoCountReply(iId, szKind);
		}

		if (strcmp(szName, "get_info_type") == 0)
		{
			const char* szKind = getCanonicalInfoKind(json_object_get_string(pArgs, "kind"));
			JSON_Value* pInfoValue = json_object_get_value(pArgs, "value");
			int iInfo = -1;
			if (pInfoValue == NULL)
			{
				pInfoValue = json_object_get_value(pArgs, "type");
			}
			if (pInfoValue == NULL)
			{
				pInfoValue = json_object_get_value(pArgs, "id");
			}
			if (szKind == NULL)
			{
				return makeErrorReply(iId, "bad_kind", "kind is missing or unsupported");
			}
			iInfo = getInfoTypeFromValue(pInfoValue);
			if (pInfoValue == NULL || iInfo < 0 || iInfo >= getInfoCountForKind(szKind))
			{
				return makeErrorReply(iId, "bad_info", "value is missing or out of range for kind");
			}
			return makeInfoTypeReply(iId, szKind, iInfo);
		}

		if (strcmp(szName, "list_info_types") == 0)
		{
			const char* szKind = getCanonicalInfoKind(json_object_get_string(pArgs, "kind"));
			if (szKind == NULL)
			{
				return makeErrorReply(iId, "bad_kind", "kind is missing or unsupported");
			}
			return makeInfoTypesListReply(iId, szKind);
		}

		if (strcmp(szName, "get_game_option_state") == 0)
		{
			JSON_Value* pOptionValue = json_object_get_value(pArgs, "option");
			int iOption = getInfoTypeFromValue(pOptionValue);
			if (pOptionValue == NULL || iOption < 0 || iOption >= GC.getNumGameOptionInfos())
			{
				return makeErrorReply(iId, "bad_option", "option is missing or out of range");
			}
			return makeGameOptionStateReply(iId, iOption);
		}

		if (strcmp(szName, "get_multiplayer_option_state") == 0)
		{
			JSON_Value* pOptionValue = json_object_get_value(pArgs, "option");
			int iOption = getInfoTypeFromValue(pOptionValue);
			if (pOptionValue == NULL || iOption < 0 || iOption >= GC.getNumMPOptionInfos())
			{
				return makeErrorReply(iId, "bad_option", "option is missing or out of range");
			}
			return makeMultiplayerOptionStateReply(iId, iOption);
		}

		if (strcmp(szName, "get_force_control_state") == 0)
		{
			JSON_Value* pControlValue = json_object_get_value(pArgs, "control");
			int iControl = getInfoTypeFromValue(pControlValue);
			if (pControlValue == NULL || iControl < 0 || iControl >= GC.getNumForceControlInfos())
			{
				return makeErrorReply(iId, "bad_control", "control is missing or out of range");
			}
			return makeForceControlStateReply(iId, iControl);
		}

		if (strcmp(szName, "get_player_gold") == 0)
		{
			int iPlayer = -1;
			if (!getPlayerArg(pArgs, iPlayer))
			{
				return makeErrorReply(iId, "bad_player", "player is missing or out of range");
			}
			CvString szReply;
			setReplyResultInt(szReply, iId, "gold", GET_PLAYER((PlayerTypes)iPlayer).getGold());
			return szReply;
		}

		if (strcmp(szName, "get_player_state") == 0)
		{
			int iPlayer = -1;
			if (!getPlayerArg(pArgs, iPlayer))
			{
				return makeErrorReply(iId, "bad_player", "player is missing or out of range");
			}
			return makePlayerStateReply(iId, iPlayer);
		}

		if (strcmp(szName, "list_players") == 0)
		{
			return makePlayersListReply(iId);
		}

		if (strcmp(szName, "get_player_options") == 0)
		{
			int iPlayer = -1;
			if (!getPlayerArg(pArgs, iPlayer))
			{
				return makeErrorReply(iId, "bad_player", "player is missing or out of range");
			}
			return makePlayerOptionsReply(iId, iPlayer);
		}

		if (strcmp(szName, "get_player_economy_state") == 0)
		{
			int iPlayer = -1;
			if (!getPlayerArg(pArgs, iPlayer))
			{
				return makeErrorReply(iId, "bad_player", "player is missing or out of range");
			}
			return makePlayerEconomyStateReply(iId, iPlayer);
		}

		if (strcmp(szName, "get_player_gold_per_turn_state") == 0)
		{
			int iPlayer = -1;
			int iOtherPlayer = -1;
			if (!getPlayerArg(pArgs, iPlayer))
			{
				return makeErrorReply(iId, "bad_player", "player is missing or out of range");
			}
			if (!getInt(pArgs, "other_player", iOtherPlayer) || !validPlayer(iOtherPlayer))
			{
				return makeErrorReply(iId, "bad_other_player", "other_player is missing or out of range");
			}
			return makePlayerGoldPerTurnStateReply(iId, iPlayer, iOtherPlayer);
		}

		if (strcmp(szName, "get_team_tech_state") == 0)
		{
			int iTeam = -1;
			if (!getInt(pArgs, "team", iTeam) || !validTeam(iTeam))
			{
				return makeErrorReply(iId, "bad_team", "team is missing or out of range");
			}
			int iTech = getInfoTypeFromValue(json_object_get_value(pArgs, "tech"));
			if (iTech < 0 || iTech >= GC.getNumTechInfos())
			{
				return makeErrorReply(iId, "bad_tech", "tech is missing or out of range");
			}
			return makeTeamTechStateReply(iId, iTeam, iTech);
		}

		if (strcmp(szName, "get_team_state") == 0)
		{
			int iTeam = -1;
			if (!getInt(pArgs, "team", iTeam) || !validEverTeam(iTeam))
			{
				return makeErrorReply(iId, "bad_team", "team is missing, out of range, or has never existed");
			}
			return makeTeamStateReply(iId, iTeam);
		}

		if (strcmp(szName, "list_teams") == 0)
		{
			return makeTeamsListReply(iId);
		}

		if (strcmp(szName, "get_team_relation_state") == 0)
		{
			int iTeam = -1;
			int iOtherTeam = -1;
			if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
			{
				return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, out of range, or have never existed");
			}
			return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
		}

		if (strcmp(szName, "get_map_state") == 0)
		{
			JSON_Object* pResult = NULL;
			JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
			json_object_set_number(pResult, "width", GC.getMapINLINE().getGridWidthINLINE());
			json_object_set_number(pResult, "height", GC.getMapINLINE().getGridHeightINLINE());
			json_object_set_number(pResult, "plots", GC.getMapINLINE().numPlotsINLINE());
			json_object_set_number(pResult, "land_plots", GC.getMapINLINE().getLandPlots());
			return serializeAndFree(pValue);
		}

		if (strcmp(szName, "get_plot_state") == 0)
		{
			int iX = -1;
			int iY = -1;
			CvPlot* pPlot = NULL;
			if (!getPlotArgs(pArgs, iX, iY, pPlot))
			{
				return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
			}
			return makePlotStateReply(iId, pPlot);
		}

		if (strcmp(szName, "get_plot_culture_state") == 0)
		{
			int iX = -1;
			int iY = -1;
			int iPlayer = -1;
			CvPlot* pPlot = NULL;
			if (!getPlotArgs(pArgs, iX, iY, pPlot))
			{
				return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
			}
			if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
			{
				return makeErrorReply(iId, "bad_player", "player is missing or out of range");
			}
			return makePlotCultureStateReply(iId, pPlot, iPlayer);
		}

		if (strcmp(szName, "get_plot_visibility_state") == 0)
		{
			int iX = -1;
			int iY = -1;
			int iTeam = -1;
			int iDebug = 0;
			CvPlot* pPlot = NULL;
			if (!getPlotArgs(pArgs, iX, iY, pPlot))
			{
				return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
			}
			if (!getInt(pArgs, "team", iTeam) || !validTeam(iTeam))
			{
				return makeErrorReply(iId, "bad_team", "team is missing or out of range");
			}
			getInt(pArgs, "debug", iDebug);
			return makePlotVisibilityStateReply(iId, pPlot, iTeam, iDebug);
		}

		if (strcmp(szName, "get_city_state") == 0)
		{
			int iPlayer = -1;
			int iCity = -1;
			CvCity* pCity = NULL;
			if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
			{
				return makeErrorReply(iId, "bad_city", "city is missing or not found");
			}
			return makeCityStateReply(iId, pCity);
		}

		if (strcmp(szName, "get_city_detail_state") == 0)
		{
			int iPlayer = -1;
			int iCity = -1;
			CvCity* pCity = NULL;
			if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
			{
				return makeErrorReply(iId, "bad_city", "city is missing or not found");
			}
			return makeCityDetailStateReply(iId, pCity);
		}

		if (strcmp(szName, "get_city_production_options") == 0)
		{
			int iPlayer = -1;
			int iCity = -1;
			int iContinueCurrent = 0;
			int iTestVisible = 0;
			int iIgnoreCost = 0;
			int iIgnoreUpgrades = 0;
			CvCity* pCity = NULL;
			if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
			{
				return makeErrorReply(iId, "bad_city", "city is missing or not found");
			}
			getInt(pArgs, "continue_current", iContinueCurrent);
			getInt(pArgs, "test_visible", iTestVisible);
			getInt(pArgs, "ignore_cost", iIgnoreCost);
			getInt(pArgs, "ignore_upgrades", iIgnoreUpgrades);
			return makeCityProductionOptionsReply(iId, pCity, iContinueCurrent, iTestVisible, iIgnoreCost, iIgnoreUpgrades);
		}

		if (strcmp(szName, "get_city_building_state") == 0)
		{
			int iPlayer = -1;
			int iCity = -1;
			CvCity* pCity = NULL;
			JSON_Value* pBuildingValue = json_object_get_value(pArgs, "building");
			int iBuilding = getInfoTypeFromValue(pBuildingValue);
			if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
			{
				return makeErrorReply(iId, "bad_city", "city is missing or not found");
			}
			if (pBuildingValue == NULL || iBuilding < 0 || iBuilding >= GC.getNumBuildingInfos())
			{
				return makeErrorReply(iId, "bad_building", "building is missing or out of range");
			}
			return makeCityBuildingStateReply(iId, pCity, iBuilding);
		}

		if (strcmp(szName, "get_city_religion_state") == 0)
		{
			int iPlayer = -1;
			int iCity = -1;
			CvCity* pCity = NULL;
			JSON_Value* pReligionValue = json_object_get_value(pArgs, "religion");
			int iReligion = getInfoTypeFromValue(pReligionValue);
			if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
			{
				return makeErrorReply(iId, "bad_city", "city is missing or not found");
			}
			if (pReligionValue == NULL || iReligion < 0 || iReligion >= GC.getNumReligionInfos())
			{
				return makeErrorReply(iId, "bad_religion", "religion is missing or out of range");
			}
			return makeCityReligionStateReply(iId, pCity, iReligion);
		}

		if (strcmp(szName, "get_city_corporation_state") == 0)
		{
			int iPlayer = -1;
			int iCity = -1;
			CvCity* pCity = NULL;
			JSON_Value* pCorporationValue = json_object_get_value(pArgs, "corporation");
			int iCorporation = getInfoTypeFromValue(pCorporationValue);
			if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
			{
				return makeErrorReply(iId, "bad_city", "city is missing or not found");
			}
			if (pCorporationValue == NULL || iCorporation < 0 || iCorporation >= GC.getNumCorporationInfos())
			{
				return makeErrorReply(iId, "bad_corporation", "corporation is missing or out of range");
			}
			return makeCityCorporationStateReply(iId, pCity, iCorporation);
		}

		if (strcmp(szName, "get_city_building_class_change") == 0)
		{
			int iPlayer = -1;
			int iCity = -1;
			CvCity* pCity = NULL;
			JSON_Value* pBuildingClassValue = json_object_get_value(pArgs, "building_class");
			int iBuildingClass = getInfoTypeFromValue(pBuildingClassValue);
			if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
			{
				return makeErrorReply(iId, "bad_city", "city is missing or not found");
			}
			if (pBuildingClassValue == NULL || iBuildingClass < 0 || iBuildingClass >= GC.getNumBuildingClassInfos())
			{
				return makeErrorReply(iId, "bad_building_class", "building_class is missing or out of range");
			}
			return makeCityBuildingClassChangeReply(iId, pCity, iBuildingClass);
		}

		if (strcmp(szName, "list_player_cities") == 0)
		{
			int iPlayer = -1;
			if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
			{
				return makeErrorReply(iId, "bad_player", "player is missing or out of range");
			}
			return makeCitiesListReply(iId, iPlayer);
		}

		if (strcmp(szName, "get_unit_state") == 0)
		{
			int iPlayer = -1;
			int iUnit = -1;
			CvUnit* pUnit = NULL;
			if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
			{
				return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
			}
			return makeUnitStateReply(iId, pUnit);
		}

		if (strcmp(szName, "get_unit_detail_state") == 0)
		{
			int iPlayer = -1;
			int iUnit = -1;
			CvUnit* pUnit = NULL;
			if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
			{
				return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
			}
			return makeUnitDetailStateReply(iId, pUnit);
		}

		if (strcmp(szName, "get_unit_promotion_state") == 0)
		{
			int iPlayer = -1;
			int iUnit = -1;
			CvUnit* pUnit = NULL;
			if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
			{
				return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
			}
			JSON_Value* pPromotionValue = json_object_get_value(pArgs, "promotion");
			int iPromotion = getInfoTypeFromValue(pPromotionValue);
			if (pPromotionValue == NULL || iPromotion < 0 || iPromotion >= GC.getNumPromotionInfos())
			{
				return makeErrorReply(iId, "bad_promotion", "promotion is missing or out of range");
			}
			return makeUnitPromotionStateReply(iId, pUnit, iPromotion);
		}

		if (strcmp(szName, "list_player_units") == 0)
		{
			int iPlayer = -1;
			if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
			{
				return makeErrorReply(iId, "bad_player", "player is missing or out of range");
			}
			return makeUnitsListReply(iId, iPlayer);
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

	CvString handleChangePlayerGold(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iChange = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}

		GET_PLAYER((PlayerTypes)iPlayer).changeGold(iChange);
		markGameDataDirty();
		return makePlayerStateReply(iId, iPlayer);
	}

	CvString handleSetPlayerAlive(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iAlive = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "alive", iAlive))
		{
			return makeErrorReply(iId, "bad_alive", "alive is missing");
		}

		GET_PLAYER((PlayerTypes)iPlayer).setAlive(iAlive != 0);
		markGameDataDirty();
		return makePlayerStateReply(iId, iPlayer);
	}

	CvString handleSetPlayerPlayable(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iPlayable = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "playable", iPlayable))
		{
			return makeErrorReply(iId, "bad_playable", "playable is missing");
		}

		GET_PLAYER((PlayerTypes)iPlayer).setPlayable(iPlayable != 0);
		markGameDataDirty();
		return makePlayerStateReply(iId, iPlayer);
	}

	CvString handleSetPlayerCurrentEra(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iEra = getInfoTypeFromValue(json_object_get_value(pArgs, "era"));
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (iEra < 0 || iEra >= GC.getNumEraInfos())
		{
			return makeErrorReply(iId, "bad_era", "era is missing or out of range");
		}

		GET_PLAYER((PlayerTypes)iPlayer).setCurrentEra((EraTypes)iEra);
		markGameDataDirty();
		return makePlayerStateReply(iId, iPlayer);
	}

	CvString handleSetPlayerPersonality(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iLeader = getInfoTypeFromValue(json_object_get_value(pArgs, "leader"));
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (iLeader < 0 || iLeader >= GC.getNumLeaderHeadInfos())
		{
			return makeErrorReply(iId, "bad_leader", "leader is missing or out of range");
		}

		GET_PLAYER((PlayerTypes)iPlayer).setPersonalityType((LeaderHeadTypes)iLeader);
		markGameDataDirty();
		return makePlayerStateReply(iId, iPlayer);
	}

	CvString handleSetPlayerParent(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iParent = NO_PLAYER;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "parent", iParent) || (iParent != NO_PLAYER && !validPlayer(iParent)))
		{
			return makeErrorReply(iId, "bad_parent", "parent is missing or out of range");
		}

		GET_PLAYER((PlayerTypes)iPlayer).setParent((PlayerTypes)iParent);
		markGameDataDirty();
		return makePlayerStateReply(iId, iPlayer);
	}

	CvString handleSetPlayerAdvancedStartPoints(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iValue = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < -1)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or less than -1");
		}

		GET_PLAYER((PlayerTypes)iPlayer).setAdvancedStartPoints(iValue);
		markGameDataDirty();
		return makePlayerEconomyStateReply(iId, iPlayer);
	}

	CvString handleChangePlayerAdvancedStartPoints(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iChange = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (GET_PLAYER((PlayerTypes)iPlayer).getAdvancedStartPoints() + iChange < -1)
		{
			return makeErrorReply(iId, "bad_value", "advanced_start_points cannot be reduced below -1");
		}

		GET_PLAYER((PlayerTypes)iPlayer).changeAdvancedStartPoints(iChange);
		markGameDataDirty();
		return makePlayerEconomyStateReply(iId, iPlayer);
	}

	CvString handleChangePlayerGoldenAgeTurns(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iChange = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (GET_PLAYER((PlayerTypes)iPlayer).getGoldenAgeTurns() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "golden_age_turns cannot be reduced below zero");
		}

		GET_PLAYER((PlayerTypes)iPlayer).changeGoldenAgeTurns(iChange);
		markGameDataDirty();
		return makePlayerEconomyStateReply(iId, iPlayer);
	}

	CvString handleChangePlayerNumUnitGoldenAges(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iChange = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (GET_PLAYER((PlayerTypes)iPlayer).getNumUnitGoldenAges() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "num_unit_golden_ages cannot be reduced below zero");
		}

		GET_PLAYER((PlayerTypes)iPlayer).changeNumUnitGoldenAges(iChange);
		markGameDataDirty();
		return makePlayerEconomyStateReply(iId, iPlayer);
	}

	CvString handleChangePlayerAnarchyTurns(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iChange = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (GET_PLAYER((PlayerTypes)iPlayer).getAnarchyTurns() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "anarchy_turns cannot be reduced below zero");
		}

		GET_PLAYER((PlayerTypes)iPlayer).changeAnarchyTurns(iChange);
		markGameDataDirty();
		return makePlayerEconomyStateReply(iId, iPlayer);
	}

	CvString handleChangePlayerStrikeTurns(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iChange = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (GET_PLAYER((PlayerTypes)iPlayer).getStrikeTurns() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "strike_turns cannot be reduced below zero");
		}

		GET_PLAYER((PlayerTypes)iPlayer).changeStrikeTurns(iChange);
		markGameDataDirty();
		return makePlayerEconomyStateReply(iId, iPlayer);
	}

	CvString handleSetPlayerCombatExperience(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iValue = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		GET_PLAYER((PlayerTypes)iPlayer).setCombatExperience(iValue);
		markGameDataDirty();
		return makePlayerEconomyStateReply(iId, iPlayer);
	}

	CvString handleChangePlayerCombatExperience(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iChange = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (GET_PLAYER((PlayerTypes)iPlayer).getCombatExperience() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "combat_experience cannot be reduced below zero");
		}

		GET_PLAYER((PlayerTypes)iPlayer).changeCombatExperience(iChange);
		markGameDataDirty();
		return makePlayerEconomyStateReply(iId, iPlayer);
	}

	CvString handleSetPlayerCommercePercent(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iValue = 0;
		int iCommerce = getCommerceTypeFromValue(json_object_get_value(pArgs, "commerce"));
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (iCommerce < 0 || iCommerce >= NUM_COMMERCE_TYPES)
		{
			return makeErrorReply(iId, "bad_commerce", "commerce is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0 || iValue > 100)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or not between 0 and 100");
		}

		GET_PLAYER((PlayerTypes)iPlayer).setCommercePercent((CommerceTypes)iCommerce, iValue);
		markGameDataDirty();
		return makePlayerEconomyStateReply(iId, iPlayer);
	}

	CvString handleChangePlayerCommercePercent(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iChange = 0;
		int iCommerce = getCommerceTypeFromValue(json_object_get_value(pArgs, "commerce"));
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (iCommerce < 0 || iCommerce >= NUM_COMMERCE_TYPES)
		{
			return makeErrorReply(iId, "bad_commerce", "commerce is missing or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (GET_PLAYER((PlayerTypes)iPlayer).getCommercePercent((CommerceTypes)iCommerce) + iChange < 0 ||
			GET_PLAYER((PlayerTypes)iPlayer).getCommercePercent((CommerceTypes)iCommerce) + iChange > 100)
		{
			return makeErrorReply(iId, "bad_value", "commerce percent cannot be changed outside 0..100");
		}

		GET_PLAYER((PlayerTypes)iPlayer).changeCommercePercent((CommerceTypes)iCommerce, iChange);
		markGameDataDirty();
		return makePlayerEconomyStateReply(iId, iPlayer);
	}

	CvString handleChangePlayerCommerceRateModifier(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iChange = 0;
		int iCommerce = getCommerceTypeFromValue(json_object_get_value(pArgs, "commerce"));
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (iCommerce < 0 || iCommerce >= NUM_COMMERCE_TYPES)
		{
			return makeErrorReply(iId, "bad_commerce", "commerce is missing or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}

		GET_PLAYER((PlayerTypes)iPlayer).changeCommerceRateModifier((CommerceTypes)iCommerce, iChange);
		markGameDataDirty();
		return makePlayerEconomyStateReply(iId, iPlayer);
	}

	CvString handleSetPlayerGoldPerTurnByPlayer(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iOtherPlayer = -1;
		int iValue = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "other_player", iOtherPlayer) || !validPlayer(iOtherPlayer))
		{
			return makeErrorReply(iId, "bad_other_player", "other_player is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue))
		{
			return makeErrorReply(iId, "bad_value", "value is missing");
		}

		GET_PLAYER((PlayerTypes)iPlayer).changeGoldPerTurnByPlayer((PlayerTypes)iOtherPlayer,
			iValue - GET_PLAYER((PlayerTypes)iPlayer).getGoldPerTurnByPlayer((PlayerTypes)iOtherPlayer));
		markGameDataDirty();
		return makePlayerGoldPerTurnStateReply(iId, iPlayer, iOtherPlayer);
	}

	CvString handleChangePlayerGoldPerTurnByPlayer(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iOtherPlayer = -1;
		int iChange = 0;
		if (!getPlayerArg(pArgs, iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "other_player", iOtherPlayer) || !validPlayer(iOtherPlayer))
		{
			return makeErrorReply(iId, "bad_other_player", "other_player is missing or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}

		GET_PLAYER((PlayerTypes)iPlayer).changeGoldPerTurnByPlayer((PlayerTypes)iOtherPlayer, iChange);
		markGameDataDirty();
		return makePlayerGoldPerTurnStateReply(iId, iPlayer, iOtherPlayer);
	}

	CvString handleSetCityPopulation(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iValue = 0;
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 1)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or less than one");
		}

		pCity->setPopulation(iValue);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleChangeCityPopulation(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iChange = 0;
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (pCity->getPopulation() + iChange < 1)
		{
			return makeErrorReply(iId, "bad_value", "population cannot be reduced below one");
		}

		pCity->changePopulation(iChange);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleSetCityCulture(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iCulturePlayer = -1;
		int iValue = 0;
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (!getInt(pArgs, "culture_player", iCulturePlayer))
		{
			iCulturePlayer = pCity->getOwnerINLINE();
		}
		if (!validPlayer(iCulturePlayer))
		{
			return makeErrorReply(iId, "bad_player", "culture_player is out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pCity->setCulture((PlayerTypes)iCulturePlayer, iValue, true, true);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleSetCityProduction(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iValue = 0;
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pCity->setProduction(iValue);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleChangeCityProduction(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iChange = 0;
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (pCity->getProduction() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "production cannot be reduced below zero");
		}

		pCity->changeProduction(iChange);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleSetCityUnitProduction(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iValue = 0;
		int iUnit = getInfoTypeFromValue(json_object_get_value(pArgs, "unit_type"));
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (iUnit < 0 || iUnit >= GC.getNumUnitInfos())
		{
			return makeErrorReply(iId, "bad_unit_type", "unit_type is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pCity->setUnitProduction((UnitTypes)iUnit, iValue);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleSetCityBuildingProduction(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iValue = 0;
		int iBuilding = getInfoTypeFromValue(json_object_get_value(pArgs, "building_type"));
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (iBuilding < 0 || iBuilding >= GC.getNumBuildingInfos())
		{
			return makeErrorReply(iId, "bad_building_type", "building_type is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pCity->setBuildingProduction((BuildingTypes)iBuilding, iValue);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleSetCityProjectProduction(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iValue = 0;
		int iProject = getInfoTypeFromValue(json_object_get_value(pArgs, "project_type"));
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (iProject < 0 || iProject >= GC.getNumProjectInfos())
		{
			return makeErrorReply(iId, "bad_project_type", "project_type is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pCity->setProjectProduction((ProjectTypes)iProject, iValue);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handlePushCityOrder(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iOrder = getOrderTypeFromValue(json_object_get_value(pArgs, "order"));
		int iData1 = getInfoTypeFromValue(json_object_get_value(pArgs, "data1"));
		int iData2 = -1;
		int iSave = 1;
		int iPop = 0;
		int iAppend = 0;
		int iForce = 0;
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (iOrder < 0 || iOrder >= NUM_ORDER_TYPES)
		{
			return makeErrorReply(iId, "bad_order", "order is missing or out of range");
		}
		getInt(pArgs, "data2", iData2);
		getInt(pArgs, "save", iSave);
		getInt(pArgs, "pop", iPop);
		getInt(pArgs, "append", iAppend);
		getInt(pArgs, "force", iForce);

		if (iOrder == ORDER_TRAIN && (iData1 < 0 || iData1 >= GC.getNumUnitInfos()))
		{
			return makeErrorReply(iId, "bad_data1", "data1 unit_type is missing or out of range");
		}
		if (iOrder == ORDER_TRAIN && (iData2 < -1 || iData2 >= (int)GC.getUnitAIInfo().size()))
		{
			return makeErrorReply(iId, "bad_data2", "data2 unit_ai is out of range");
		}
		if (iOrder == ORDER_CONSTRUCT && (iData1 < 0 || iData1 >= GC.getNumBuildingInfos()))
		{
			return makeErrorReply(iId, "bad_data1", "data1 building_type is missing or out of range");
		}
		if (iOrder == ORDER_CREATE && (iData1 < 0 || iData1 >= GC.getNumProjectInfos()))
		{
			return makeErrorReply(iId, "bad_data1", "data1 project_type is missing or out of range");
		}
		if (iOrder == ORDER_MAINTAIN && (iData1 < 0 || iData1 >= GC.getNumProcessInfos()))
		{
			return makeErrorReply(iId, "bad_data1", "data1 process_type is missing or out of range");
		}

		pCity->pushOrder((OrderTypes)iOrder, iData1, iData2, iSave != 0, iPop != 0, iAppend != 0, iForce != 0);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleClearCityOrderQueue(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}

		pCity->clearOrderQueue();
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handlePopCityOrder(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iIndex = 0;
		int iFinish = 0;
		int iChoose = 0;
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		getInt(pArgs, "index", iIndex);
		getInt(pArgs, "finish", iFinish);
		getInt(pArgs, "choose", iChoose);
		if (iIndex < 0 || iIndex >= pCity->getOrderQueueLength())
		{
			return makeErrorReply(iId, "bad_index", "index is out of range");
		}

		pCity->popOrder(iIndex, iFinish != 0, iChoose != 0);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleSetCityOccupationTimer(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iValue = 0;
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pCity->setOccupationTimer(iValue);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleChangeCityOccupationTimer(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iChange = 0;
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (pCity->getOccupationTimer() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "occupation_timer cannot be reduced below zero");
		}

		pCity->changeOccupationTimer(iChange);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleChangeCityHurryAngerTimer(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iChange = 0;
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (pCity->getHurryAngerTimer() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "hurry_anger_timer cannot be reduced below zero");
		}

		pCity->changeHurryAngerTimer(iChange);
		markGameDataDirty();
		return makeCityStateReply(iId, pCity);
	}

	CvString handleSetCityRealBuilding(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iValue = 0;
		JSON_Value* pBuildingValue = json_object_get_value(pArgs, "building");
		int iBuilding = getInfoTypeFromValue(pBuildingValue);
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (pBuildingValue == NULL || iBuilding < 0 || iBuilding >= GC.getNumBuildingInfos())
		{
			return makeErrorReply(iId, "bad_building", "building is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pCity->setNumRealBuilding((BuildingTypes)iBuilding, iValue);
		markGameDataDirty();
		return makeCityBuildingStateReply(iId, pCity, iBuilding);
	}

	CvString handleSetCityFreeBuilding(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iValue = 0;
		JSON_Value* pBuildingValue = json_object_get_value(pArgs, "building");
		int iBuilding = getInfoTypeFromValue(pBuildingValue);
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (pBuildingValue == NULL || iBuilding < 0 || iBuilding >= GC.getNumBuildingInfos())
		{
			return makeErrorReply(iId, "bad_building", "building is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pCity->setNumFreeBuilding((BuildingTypes)iBuilding, iValue);
		markGameDataDirty();
		return makeCityBuildingStateReply(iId, pCity, iBuilding);
	}

	CvString handleSetCityReligion(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iHas = 0;
		int iAnnounce = 0;
		int iArrows = 1;
		JSON_Value* pReligionValue = json_object_get_value(pArgs, "religion");
		int iReligion = getInfoTypeFromValue(pReligionValue);
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (pReligionValue == NULL || iReligion < 0 || iReligion >= GC.getNumReligionInfos())
		{
			return makeErrorReply(iId, "bad_religion", "religion is missing or out of range");
		}
		if (!getInt(pArgs, "has", iHas))
		{
			return makeErrorReply(iId, "bad_has", "has is missing");
		}
		getInt(pArgs, "announce", iAnnounce);
		getInt(pArgs, "arrows", iArrows);

		pCity->setHasReligion((ReligionTypes)iReligion, iHas != 0, iAnnounce != 0, iArrows != 0);
		markGameDataDirty();
		return makeCityReligionStateReply(iId, pCity, iReligion);
	}

	CvString handleSetCityCorporation(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iHas = 0;
		int iAnnounce = 0;
		int iArrows = 1;
		JSON_Value* pCorporationValue = json_object_get_value(pArgs, "corporation");
		int iCorporation = getInfoTypeFromValue(pCorporationValue);
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (pCorporationValue == NULL || iCorporation < 0 || iCorporation >= GC.getNumCorporationInfos())
		{
			return makeErrorReply(iId, "bad_corporation", "corporation is missing or out of range");
		}
		if (!getInt(pArgs, "has", iHas))
		{
			return makeErrorReply(iId, "bad_has", "has is missing");
		}
		getInt(pArgs, "announce", iAnnounce);
		getInt(pArgs, "arrows", iArrows);

		pCity->setHasCorporation((CorporationTypes)iCorporation, iHas != 0, iAnnounce != 0, iArrows != 0);
		markGameDataDirty();
		return makeCityCorporationStateReply(iId, pCity, iCorporation);
	}

	CvString handleSetCityBuildingHappinessChange(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iValue = 0;
		JSON_Value* pBuildingClassValue = json_object_get_value(pArgs, "building_class");
		int iBuildingClass = getInfoTypeFromValue(pBuildingClassValue);
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (pBuildingClassValue == NULL || iBuildingClass < 0 || iBuildingClass >= GC.getNumBuildingClassInfos())
		{
			return makeErrorReply(iId, "bad_building_class", "building_class is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue))
		{
			return makeErrorReply(iId, "bad_value", "value is missing");
		}

		pCity->setBuildingHappyChange((BuildingClassTypes)iBuildingClass, iValue);
		markGameDataDirty();
		return makeCityBuildingClassChangeReply(iId, pCity, iBuildingClass);
	}

	CvString handleSetCityBuildingHealthChange(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iCity = -1;
		int iValue = 0;
		JSON_Value* pBuildingClassValue = json_object_get_value(pArgs, "building_class");
		int iBuildingClass = getInfoTypeFromValue(pBuildingClassValue);
		CvCity* pCity = NULL;
		if (!getCityArgs(pArgs, iPlayer, iCity, pCity))
		{
			return makeErrorReply(iId, "bad_city", "city is missing or not found");
		}
		if (pBuildingClassValue == NULL || iBuildingClass < 0 || iBuildingClass >= GC.getNumBuildingClassInfos())
		{
			return makeErrorReply(iId, "bad_building_class", "building_class is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue))
		{
			return makeErrorReply(iId, "bad_value", "value is missing");
		}

		pCity->setBuildingHealthChange((BuildingClassTypes)iBuildingClass, iValue);
		markGameDataDirty();
		return makeCityBuildingClassChangeReply(iId, pCity, iBuildingClass);
	}

	CvString handleSetPlotOwner(int iId, JSON_Object* pArgs)
	{
		int iX = -1;
		int iY = -1;
		int iOwner = NO_PLAYER;
		int iCheckUnits = 1;
		int iUpdatePlotGroup = 1;
		CvPlot* pPlot = NULL;
		if (!getPlotArgs(pArgs, iX, iY, pPlot))
		{
			return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
		}
		if (!getInt(pArgs, "owner", iOwner) || (iOwner != NO_PLAYER && !validPlayer(iOwner)))
		{
			return makeErrorReply(iId, "bad_owner", "owner is missing or out of range");
		}
		getInt(pArgs, "check_units", iCheckUnits);
		getInt(pArgs, "update_plot_group", iUpdatePlotGroup);

		pPlot->setOwner((PlayerTypes)iOwner, iCheckUnits != 0, iUpdatePlotGroup != 0);
		markGameDataDirty();
		return makePlotStateReply(iId, pPlot);
	}

	CvString handleSetPlotTerrain(int iId, JSON_Object* pArgs)
	{
		int iX = -1;
		int iY = -1;
		int iTerrain = getInfoTypeFromValue(json_object_get_value(pArgs, "terrain"));
		int iRecalculate = 1;
		int iRebuildGraphics = 1;
		CvPlot* pPlot = NULL;
		if (!getPlotArgs(pArgs, iX, iY, pPlot))
		{
			return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
		}
		if (iTerrain < 0 || iTerrain >= GC.getNumTerrainInfos())
		{
			return makeErrorReply(iId, "bad_terrain", "terrain is missing or out of range");
		}
		getInt(pArgs, "recalculate", iRecalculate);
		getInt(pArgs, "rebuild_graphics", iRebuildGraphics);

		pPlot->setTerrainType((TerrainTypes)iTerrain, iRecalculate != 0, iRebuildGraphics != 0);
		markGameDataDirty();
		return makePlotStateReply(iId, pPlot);
	}

	CvString handleSetPlotFeature(int iId, JSON_Object* pArgs)
	{
		int iX = -1;
		int iY = -1;
		JSON_Value* pFeatureValue = json_object_get_value(pArgs, "feature");
		int iFeature = getInfoTypeFromValue(pFeatureValue);
		int iVariety = -1;
		CvPlot* pPlot = NULL;
		if (!getPlotArgs(pArgs, iX, iY, pPlot))
		{
			return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
		}
		if (pFeatureValue == NULL || iFeature < NO_FEATURE || iFeature >= GC.getNumFeatureInfos())
		{
			return makeErrorReply(iId, "bad_feature", "feature is missing or out of range");
		}
		getInt(pArgs, "variety", iVariety);

		pPlot->setFeatureType((FeatureTypes)iFeature, iVariety);
		markGameDataDirty();
		return makePlotStateReply(iId, pPlot);
	}

	CvString handleSetPlotBonus(int iId, JSON_Object* pArgs)
	{
		int iX = -1;
		int iY = -1;
		JSON_Value* pBonusValue = json_object_get_value(pArgs, "bonus");
		int iBonus = getInfoTypeFromValue(pBonusValue);
		CvPlot* pPlot = NULL;
		if (!getPlotArgs(pArgs, iX, iY, pPlot))
		{
			return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
		}
		if (pBonusValue == NULL || iBonus < NO_BONUS || iBonus >= GC.getNumBonusInfos())
		{
			return makeErrorReply(iId, "bad_bonus", "bonus is missing or out of range");
		}

		pPlot->setBonusType((BonusTypes)iBonus);
		markGameDataDirty();
		return makePlotStateReply(iId, pPlot);
	}

	CvString handleSetPlotImprovement(int iId, JSON_Object* pArgs)
	{
		int iX = -1;
		int iY = -1;
		JSON_Value* pImprovementValue = json_object_get_value(pArgs, "improvement");
		int iImprovement = getInfoTypeFromValue(pImprovementValue);
		CvPlot* pPlot = NULL;
		if (!getPlotArgs(pArgs, iX, iY, pPlot))
		{
			return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
		}
		if (pImprovementValue == NULL || iImprovement < NO_IMPROVEMENT || iImprovement >= GC.getNumImprovementInfos())
		{
			return makeErrorReply(iId, "bad_improvement", "improvement is missing or out of range");
		}

		pPlot->setImprovementType((ImprovementTypes)iImprovement);
		markGameDataDirty();
		return makePlotStateReply(iId, pPlot);
	}

	CvString handleSetPlotRoute(int iId, JSON_Object* pArgs)
	{
		int iX = -1;
		int iY = -1;
		JSON_Value* pRouteValue = json_object_get_value(pArgs, "route");
		int iRoute = getInfoTypeFromValue(pRouteValue);
		int iUpdatePlotGroup = 1;
		CvPlot* pPlot = NULL;
		if (!getPlotArgs(pArgs, iX, iY, pPlot))
		{
			return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
		}
		if (pRouteValue == NULL || iRoute < NO_ROUTE || iRoute >= GC.getNumRouteInfos())
		{
			return makeErrorReply(iId, "bad_route", "route is missing or out of range");
		}
		getInt(pArgs, "update_plot_group", iUpdatePlotGroup);

		pPlot->setRouteType((RouteTypes)iRoute, iUpdatePlotGroup != 0);
		markGameDataDirty();
		return makePlotStateReply(iId, pPlot);
	}

	CvString handleSetPlotCulture(int iId, JSON_Object* pArgs)
	{
		int iX = -1;
		int iY = -1;
		int iPlayer = -1;
		int iValue = 0;
		int iUpdate = 1;
		int iUpdatePlotGroups = 1;
		CvPlot* pPlot = NULL;
		if (!getPlotArgs(pArgs, iX, iY, pPlot))
		{
			return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
		}
		if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}
		getInt(pArgs, "update", iUpdate);
		getInt(pArgs, "update_plot_groups", iUpdatePlotGroups);

		pPlot->setCulture((PlayerTypes)iPlayer, iValue, iUpdate != 0, iUpdatePlotGroups != 0);
		markGameDataDirty();
		return makePlotStateReply(iId, pPlot);
	}

	CvString handleChangePlotCulture(int iId, JSON_Object* pArgs)
	{
		int iX = -1;
		int iY = -1;
		int iPlayer = -1;
		int iChange = 0;
		int iUpdate = 1;
		CvPlot* pPlot = NULL;
		if (!getPlotArgs(pArgs, iX, iY, pPlot))
		{
			return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
		}
		if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (pPlot->getCulture((PlayerTypes)iPlayer) + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "culture cannot be reduced below zero");
		}
		getInt(pArgs, "update", iUpdate);

		pPlot->changeCulture((PlayerTypes)iPlayer, iChange, iUpdate != 0);
		markGameDataDirty();
		return makePlotStateReply(iId, pPlot);
	}

	CvString handleSetPlotRevealed(int iId, JSON_Object* pArgs)
	{
		int iX = -1;
		int iY = -1;
		int iTeam = -1;
		int iRevealed = 0;
		int iTerrainOnly = 0;
		int iFromTeam = NO_TEAM;
		int iUpdatePlotGroup = 1;
		CvPlot* pPlot = NULL;
		if (!getPlotArgs(pArgs, iX, iY, pPlot))
		{
			return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
		}
		if (!getInt(pArgs, "team", iTeam) || !validTeam(iTeam))
		{
			return makeErrorReply(iId, "bad_team", "team is missing or out of range");
		}
		if (!getInt(pArgs, "revealed", iRevealed))
		{
			return makeErrorReply(iId, "bad_revealed", "revealed is missing");
		}
		getInt(pArgs, "terrain_only", iTerrainOnly);
		getInt(pArgs, "from_team", iFromTeam);
		getInt(pArgs, "update_plot_group", iUpdatePlotGroup);
		if (iFromTeam != NO_TEAM && !validTeam(iFromTeam))
		{
			return makeErrorReply(iId, "bad_team", "from_team is out of range");
		}

		pPlot->setRevealed((TeamTypes)iTeam, iRevealed != 0, iTerrainOnly != 0, (TeamTypes)iFromTeam, iUpdatePlotGroup != 0);
		markGameDataDirty();
		return makePlotStateReply(iId, pPlot);
	}

	CvString handleSetUnitDamage(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iValue = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0 || iValue > GC.getMAX_HIT_POINTS())
		{
			return makeErrorReply(iId, "bad_value", "value is missing or out of range");
		}

		pUnit->setDamage(iValue);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleChangeUnitDamage(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iChange = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}

		pUnit->changeDamage(iChange);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleSetUnitExperience(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iValue = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pUnit->setExperience(iValue);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleChangeUnitExperience(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iChange = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (pUnit->getExperience() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "experience cannot be reduced below zero");
		}

		pUnit->changeExperience(iChange);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleSetUnitXY(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iX = -1;
		int iY = -1;
		int iGroup = 0;
		int iUpdate = 1;
		int iShow = 0;
		int iCheckPlotVisible = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "x", iX) || !getInt(pArgs, "y", iY) || GC.getMapINLINE().plot(iX, iY) == NULL)
		{
			return makeErrorReply(iId, "bad_plot", "plot is missing or out of range");
		}
		getInt(pArgs, "group", iGroup);
		getInt(pArgs, "update", iUpdate);
		getInt(pArgs, "show", iShow);
		getInt(pArgs, "check_plot_visible", iCheckPlotVisible);

		pUnit->setXY(iX, iY, iGroup != 0, iUpdate != 0, iShow != 0, iCheckPlotVisible != 0);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleSetUnitMoves(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iValue = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pUnit->setMoves(iValue);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleChangeUnitMoves(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iChange = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (pUnit->getMoves() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "moves cannot be reduced below zero");
		}

		pUnit->changeMoves(iChange);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleFinishUnitMoves(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}

		pUnit->finishMoves();
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleSetUnitLevel(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iValue = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 1)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or less than one");
		}

		pUnit->setLevel(iValue);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleChangeUnitLevel(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iChange = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (pUnit->getLevel() + iChange < 1)
		{
			return makeErrorReply(iId, "bad_value", "level cannot be reduced below one");
		}

		pUnit->changeLevel(iChange);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleSetUnitFortifyTurns(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iValue = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pUnit->setFortifyTurns(iValue);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleChangeUnitFortifyTurns(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iChange = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (pUnit->getFortifyTurns() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "fortify_turns cannot be reduced below zero");
		}

		pUnit->changeFortifyTurns(iChange);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleSetUnitMadeAttack(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iValue = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "value", iValue))
		{
			return makeErrorReply(iId, "bad_value", "value is missing");
		}

		pUnit->setMadeAttack(iValue != 0);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleSetUnitBaseCombat(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iValue = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pUnit->setBaseCombatStr(iValue);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleSetUnitImmobileTimer(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iValue = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		pUnit->setImmobileTimer(iValue);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleChangeUnitImmobileTimer(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iChange = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}

		pUnit->changeImmobileTimer(iChange);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleSetUnitPromotion(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iHas = 0;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		JSON_Value* pPromotionValue = json_object_get_value(pArgs, "promotion");
		int iPromotion = getInfoTypeFromValue(pPromotionValue);
		if (pPromotionValue == NULL || iPromotion < 0 || iPromotion >= GC.getNumPromotionInfos())
		{
			return makeErrorReply(iId, "bad_promotion", "promotion is missing or out of range");
		}
		if (!getInt(pArgs, "has", iHas))
		{
			return makeErrorReply(iId, "bad_has", "has is missing");
		}

		pUnit->setHasPromotion((PromotionTypes)iPromotion, iHas != 0);
		markGameDataDirty();
		return makeUnitStateReply(iId, pUnit);
	}

	CvString handleKillUnit(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		int iUnit = -1;
		int iDelay = 0;
		int iKiller = NO_PLAYER;
		CvUnit* pUnit = NULL;
		if (!getUnitArgs(pArgs, iPlayer, iUnit, pUnit))
		{
			return makeErrorReply(iId, "bad_unit", "unit is missing or not found");
		}
		getInt(pArgs, "delay", iDelay);
		getInt(pArgs, "killer", iKiller);
		if (iKiller != NO_PLAYER && !validPlayer(iKiller))
		{
			return makeErrorReply(iId, "bad_player", "killer is out of range");
		}

		pUnit->kill(iDelay != 0, (PlayerTypes)iKiller);
		markGameDataDirty();

		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_number(pResult, "unit", iUnit);
		json_object_set_boolean(pResult, "killed", 1);
		return serializeAndFree(pValue);
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

		JSON_Object* pResult = NULL;
		JSON_Value* pValue = makeResultReplyValue(iId, &pResult);
		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_number(pResult, "unit", pUnit->getID());
		json_object_set_number(pResult, "x", pUnit->getX_INLINE());
		json_object_set_number(pResult, "y", pUnit->getY_INLINE());
		return serializeAndFree(pValue);
	}

	CvString handleSetGameTurn(int iId, JSON_Object* pArgs)
	{
		int iValue = 0;
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		GC.getGameINLINE().setGameTurn(iValue);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGameMaxTurns(int iId, JSON_Object* pArgs)
	{
		int iValue = 0;
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		GC.getGameINLINE().setMaxTurns(iValue);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleChangeGameMaxTurns(int iId, JSON_Object* pArgs)
	{
		int iChange = 0;
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (GC.getGameINLINE().getMaxTurns() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "max_turns cannot be reduced below zero");
		}

		GC.getGameINLINE().changeMaxTurns(iChange);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGameStartTurn(int iId, JSON_Object* pArgs)
	{
		int iValue = 0;
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		GC.getGameINLINE().setStartTurn(iValue);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGameStartYear(int iId, JSON_Object* pArgs)
	{
		int iValue = 0;
		if (!getInt(pArgs, "value", iValue))
		{
			return makeErrorReply(iId, "bad_value", "value is missing");
		}

		GC.getGameINLINE().setStartYear(iValue);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGameEstimateEndTurn(int iId, JSON_Object* pArgs)
	{
		int iValue = 0;
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		GC.getGameINLINE().setEstimateEndTurn(iValue);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGameTargetScore(int iId, JSON_Object* pArgs)
	{
		int iValue = 0;
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		GC.getGameINLINE().setTargetScore(iValue);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGameMaxCityElimination(int iId, JSON_Object* pArgs)
	{
		int iValue = 0;
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		GC.getGameINLINE().setMaxCityElimination(iValue);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGameAdvancedStartPoints(int iId, JSON_Object* pArgs)
	{
		int iValue = 0;
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		GC.getGameINLINE().setNumAdvancedStartPoints(iValue);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGameAIAutoPlay(int iId, JSON_Object* pArgs)
	{
		int iValue = 0;
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		GC.getGameINLINE().setAIAutoPlay(iValue);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleChangeGameAIAutoPlay(int iId, JSON_Object* pArgs)
	{
		int iChange = 0;
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}

		GC.getGameINLINE().changeAIAutoPlay(iChange);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleChangeGameNukesExploded(int iId, JSON_Object* pArgs)
	{
		int iChange = 0;
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (GC.getGameINLINE().getNukesExploded() + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "nukes_exploded cannot be reduced below zero");
		}

		GC.getGameINLINE().changeNukesExploded(iChange);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGamePausePlayer(int iId, JSON_Object* pArgs)
	{
		int iPlayer = NO_PLAYER;
		if (!getInt(pArgs, "player", iPlayer) || (iPlayer != NO_PLAYER && !validPlayer(iPlayer)))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}

		GC.getGameINLINE().setPausePlayer((PlayerTypes)iPlayer);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGameWinner(int iId, JSON_Object* pArgs)
	{
		int iTeam = NO_TEAM;
		JSON_Value* pVictoryValue = json_object_get_value(pArgs, "victory");
		int iVictory = getInfoTypeFromValue(pVictoryValue);
		getInt(pArgs, "team", iTeam);
		if (iTeam != NO_TEAM && !validEverTeam(iTeam))
		{
			return makeErrorReply(iId, "bad_team", "team is out of range or has never existed");
		}
		if (pVictoryValue == NULL || iVictory < NO_VICTORY || iVictory >= GC.getNumVictoryInfos())
		{
			return makeErrorReply(iId, "bad_victory", "victory is missing or out of range");
		}
		if ((iTeam == NO_TEAM) != (iVictory == NO_VICTORY))
		{
			return makeErrorReply(iId, "bad_winner", "team and victory must both be set or both be cleared");
		}

		GC.getGameINLINE().setWinner((TeamTypes)iTeam, (VictoryTypes)iVictory);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGameState(int iId, JSON_Object* pArgs)
	{
		int iValue = getGameStateTypeFromValue(json_object_get_value(pArgs, "value"));
		if (iValue < GAMESTATE_ON || iValue > GAMESTATE_EXTENDED)
		{
			return makeErrorReply(iId, "bad_state", "value is missing or out of range");
		}

		GC.getGameINLINE().setGameState((GameStateTypes)iValue);
		markGameDataDirty();
		return makeGameStateReply(iId);
	}

	CvString handleSetGameOption(int iId, JSON_Object* pArgs)
	{
		int iEnabled = 0;
		JSON_Value* pOptionValue = json_object_get_value(pArgs, "option");
		int iOption = getInfoTypeFromValue(pOptionValue);
		if (pOptionValue == NULL || iOption < 0 || iOption >= GC.getNumGameOptionInfos())
		{
			return makeErrorReply(iId, "bad_option", "option is missing or out of range");
		}
		if (!getInt(pArgs, "enabled", iEnabled))
		{
			return makeErrorReply(iId, "bad_enabled", "enabled is missing");
		}

		GC.getGameINLINE().setOption((GameOptionTypes)iOption, iEnabled != 0);
		markGameDataDirty();
		return makeGameOptionStateReply(iId, iOption);
	}

	CvString handleSetMultiplayerOption(int iId, JSON_Object* pArgs)
	{
		int iEnabled = 0;
		JSON_Value* pOptionValue = json_object_get_value(pArgs, "option");
		int iOption = getInfoTypeFromValue(pOptionValue);
		if (pOptionValue == NULL || iOption < 0 || iOption >= GC.getNumMPOptionInfos())
		{
			return makeErrorReply(iId, "bad_option", "option is missing or out of range");
		}
		if (!getInt(pArgs, "enabled", iEnabled))
		{
			return makeErrorReply(iId, "bad_enabled", "enabled is missing");
		}

		GC.getGameINLINE().setMPOption((MultiplayerOptionTypes)iOption, iEnabled != 0);
		markGameDataDirty();
		return makeMultiplayerOptionStateReply(iId, iOption);
	}

	CvString handleSetForceControl(int iId, JSON_Object* pArgs)
	{
		int iEnabled = 0;
		JSON_Value* pControlValue = json_object_get_value(pArgs, "control");
		int iControl = getInfoTypeFromValue(pControlValue);
		if (pControlValue == NULL || iControl < 0 || iControl >= GC.getNumForceControlInfos())
		{
			return makeErrorReply(iId, "bad_control", "control is missing or out of range");
		}
		if (!getInt(pArgs, "enabled", iEnabled))
		{
			return makeErrorReply(iId, "bad_enabled", "enabled is missing");
		}

		GC.getGameINLINE().setForceControl((ForceControlTypes)iControl, iEnabled != 0);
		markGameDataDirty();
		return makeForceControlStateReply(iId, iControl);
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

	CvString handleSetPlayerCivic(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}

		int iCivic = getInfoTypeFromValue(json_object_get_value(pArgs, "civic"));
		if (iCivic < 0 || iCivic >= GC.getNumCivicInfos())
		{
			return makeErrorReply(iId, "bad_civic", "civic is missing or out of range");
		}

		int iOption = -1;
		if (!getInt(pArgs, "civic_option", iOption))
		{
			iOption = GC.getCivicInfo((CivicTypes)iCivic).getCivicOptionType();
		}
		if (iOption < 0 || iOption >= GC.getNumCivicOptionInfos())
		{
			return makeErrorReply(iId, "bad_civic_option", "civic_option is out of range");
		}
		if (GC.getCivicInfo((CivicTypes)iCivic).getCivicOptionType() != iOption)
		{
			return makeErrorReply(iId, "bad_civic_option", "civic does not belong to civic_option");
		}

		GET_PLAYER((PlayerTypes)iPlayer).setCivics((CivicOptionTypes)iOption, (CivicTypes)iCivic);
		markGameDataDirty();
		return makePlayerOptionsReply(iId, iPlayer);
	}

	CvString handleSetPlayerStateReligion(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}

		JSON_Value* pReligion = json_object_get_value(pArgs, "religion");
		if (pReligion == NULL)
		{
			return makeErrorReply(iId, "bad_religion", "religion is missing or out of range");
		}

		int iReligion = getInfoTypeFromValue(pReligion);
		if (iReligion < NO_RELIGION || iReligion >= GC.getNumReligionInfos())
		{
			return makeErrorReply(iId, "bad_religion", "religion is missing or out of range");
		}

		GET_PLAYER((PlayerTypes)iPlayer).setLastStateReligion((ReligionTypes)iReligion);
		markGameDataDirty();
		return makePlayerOptionsReply(iId, iPlayer);
	}

	CvString handleSetPlayerResearch(int iId, JSON_Object* pArgs)
	{
		int iPlayer = -1;
		if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is missing or out of range");
		}

		int iTech = getInfoTypeFromValue(json_object_get_value(pArgs, "tech"));
		if (iTech < 0 || iTech >= GC.getNumTechInfos())
		{
			return makeErrorReply(iId, "bad_tech", "tech is missing or out of range");
		}

		bool bOk = GET_PLAYER((PlayerTypes)iPlayer).pushResearch((TechTypes)iTech, true);
		if (!bOk)
		{
			return makeErrorReply(iId, "research_rejected", "player cannot research tech");
		}

		markGameDataDirty();
		return makePlayerOptionsReply(iId, iPlayer);
	}

	CvString handleSetTeamHasTech(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iPlayer = NO_PLAYER;
		int iHas = 0;
		if (!getInt(pArgs, "team", iTeam) || !validTeam(iTeam))
		{
			return makeErrorReply(iId, "bad_team", "team is missing or out of range");
		}
		getInt(pArgs, "player", iPlayer);
		if (iPlayer != NO_PLAYER && !validPlayer(iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is out of range");
		}
		if (!getInt(pArgs, "has", iHas))
		{
			return makeErrorReply(iId, "bad_has", "has is missing");
		}

		int iTech = getInfoTypeFromValue(json_object_get_value(pArgs, "tech"));
		if (iTech < 0 || iTech >= GC.getNumTechInfos())
		{
			return makeErrorReply(iId, "bad_tech", "tech is missing or out of range");
		}

		GET_TEAM((TeamTypes)iTeam).setHasTech((TechTypes)iTech, iHas != 0, (PlayerTypes)iPlayer, false, true);
		markGameDataDirty();
		return makeTeamTechStateReply(iId, iTeam, iTech);
	}

	CvString handleChangeTeamResearchProgress(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iPlayer = NO_PLAYER;
		int iChange = 0;
		if (!getInt(pArgs, "team", iTeam) || !validTeam(iTeam))
		{
			return makeErrorReply(iId, "bad_team", "team is missing or out of range");
		}
		getInt(pArgs, "player", iPlayer);
		if (iPlayer != NO_PLAYER && !validPlayer(iPlayer))
		{
			return makeErrorReply(iId, "bad_player", "player is out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}

		int iTech = getInfoTypeFromValue(json_object_get_value(pArgs, "tech"));
		if (iTech < 0 || iTech >= GC.getNumTechInfos())
		{
			return makeErrorReply(iId, "bad_tech", "tech is missing or out of range");
		}

		GET_TEAM((TeamTypes)iTeam).changeResearchProgress((TechTypes)iTech, iChange, (PlayerTypes)iPlayer);
		markGameDataDirty();
		return makeTeamTechStateReply(iId, iTeam, iTech);
	}

	CvString handleMeetTeam(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iNewDiplo = 0;
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		getInt(pArgs, "new_diplo", iNewDiplo);

		GET_TEAM((TeamTypes)iTeam).meet((TeamTypes)iOtherTeam, iNewDiplo != 0);
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleDeclareWar(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iNewDiplo = 0;
		int iWarPlan = getWarPlanTypeFromValue(json_object_get_value(pArgs, "war_plan"));
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		if (iWarPlan < NO_WARPLAN || iWarPlan > WARPLAN_DOGPILE)
		{
			return makeErrorReply(iId, "bad_war_plan", "war_plan is out of range");
		}
		if (!GET_TEAM((TeamTypes)iTeam).canDeclareWar((TeamTypes)iOtherTeam))
		{
			return makeErrorReply(iId, "war_rejected", "team cannot declare war on other_team");
		}
		getInt(pArgs, "new_diplo", iNewDiplo);

		GET_TEAM((TeamTypes)iTeam).declareWar((TeamTypes)iOtherTeam, iNewDiplo != 0, (WarPlanTypes)iWarPlan);
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleMakePeace(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iBumpUnits = 1;
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		getInt(pArgs, "bump_units", iBumpUnits);

		GET_TEAM((TeamTypes)iTeam).makePeace((TeamTypes)iOtherTeam, iBumpUnits != 0);
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleSetTeamOpenBorders(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iOpen = 0;
		int iReciprocal = 1;
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		if (!getInt(pArgs, "open", iOpen))
		{
			return makeErrorReply(iId, "bad_open", "open is missing");
		}
		getInt(pArgs, "reciprocal", iReciprocal);

		GET_TEAM((TeamTypes)iTeam).setOpenBorders((TeamTypes)iOtherTeam, iOpen != 0);
		if (iReciprocal != 0)
		{
			GET_TEAM((TeamTypes)iOtherTeam).setOpenBorders((TeamTypes)iTeam, iOpen != 0);
		}
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleSetTeamDefensivePact(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iPact = 0;
		int iReciprocal = 1;
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		if (!getInt(pArgs, "pact", iPact))
		{
			return makeErrorReply(iId, "bad_pact", "pact is missing");
		}
		getInt(pArgs, "reciprocal", iReciprocal);

		GET_TEAM((TeamTypes)iTeam).setDefensivePact((TeamTypes)iOtherTeam, iPact != 0);
		if (iReciprocal != 0)
		{
			GET_TEAM((TeamTypes)iOtherTeam).setDefensivePact((TeamTypes)iTeam, iPact != 0);
		}
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleSetTeamForcePeace(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iPeace = 0;
		int iReciprocal = 1;
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		if (!getInt(pArgs, "peace", iPeace))
		{
			return makeErrorReply(iId, "bad_peace", "peace is missing");
		}
		getInt(pArgs, "reciprocal", iReciprocal);

		GET_TEAM((TeamTypes)iTeam).setForcePeace((TeamTypes)iOtherTeam, iPeace != 0);
		if (iReciprocal != 0)
		{
			GET_TEAM((TeamTypes)iOtherTeam).setForcePeace((TeamTypes)iTeam, iPeace != 0);
		}
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleSetTeamPermanentWarPeace(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iPermanent = 0;
		int iReciprocal = 1;
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		if (!getInt(pArgs, "permanent", iPermanent))
		{
			return makeErrorReply(iId, "bad_permanent", "permanent is missing");
		}
		getInt(pArgs, "reciprocal", iReciprocal);

		GET_TEAM((TeamTypes)iTeam).setPermanentWarPeace((TeamTypes)iOtherTeam, iPermanent != 0);
		if (iReciprocal != 0)
		{
			GET_TEAM((TeamTypes)iOtherTeam).setPermanentWarPeace((TeamTypes)iTeam, iPermanent != 0);
		}
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleSetTeamVassal(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iVassal = 0;
		int iCapitulated = 0;
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		if (!getInt(pArgs, "vassal", iVassal))
		{
			return makeErrorReply(iId, "bad_vassal", "vassal is missing");
		}
		if (iVassal != 0 && GET_TEAM((TeamTypes)iOtherTeam).isAVassal())
		{
			return makeErrorReply(iId, "vassal_rejected", "other_team is already a vassal");
		}
		getInt(pArgs, "capitulated", iCapitulated);

		GET_TEAM((TeamTypes)iTeam).setVassal((TeamTypes)iOtherTeam, iVassal != 0, iCapitulated != 0);
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleSetTeamWarWeariness(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iValue = 0;
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		GET_TEAM((TeamTypes)iTeam).setWarWeariness((TeamTypes)iOtherTeam, iValue);
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleChangeTeamWarWeariness(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iChange = 0;
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}

		GET_TEAM((TeamTypes)iTeam).changeWarWeariness((TeamTypes)iOtherTeam, iChange);
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleSetTeamStolenVisibilityTimer(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iValue = 0;
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		if (!getInt(pArgs, "value", iValue) || iValue < 0)
		{
			return makeErrorReply(iId, "bad_value", "value is missing or negative");
		}

		GET_TEAM((TeamTypes)iTeam).setStolenVisibilityTimer((TeamTypes)iOtherTeam, iValue);
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleChangeTeamStolenVisibilityTimer(int iId, JSON_Object* pArgs)
	{
		int iTeam = -1;
		int iOtherTeam = -1;
		int iChange = 0;
		if (!getTeamRelationArgs(pArgs, iTeam, iOtherTeam))
		{
			return makeErrorReply(iId, "bad_team", "team and other_team are missing, equal, or out of range");
		}
		if (!getInt(pArgs, "change", iChange))
		{
			return makeErrorReply(iId, "bad_change", "change is missing");
		}
		if (GET_TEAM((TeamTypes)iTeam).getStolenVisibilityTimer((TeamTypes)iOtherTeam) + iChange < 0)
		{
			return makeErrorReply(iId, "bad_value", "stolen_visibility_timer cannot be reduced below zero");
		}

		GET_TEAM((TeamTypes)iTeam).changeStolenVisibilityTimer((TeamTypes)iOtherTeam, iChange);
		markGameDataDirty();
		return makeTeamRelationStateReply(iId, iTeam, iOtherTeam);
	}

	CvString handleCommand(int iId, const char* szName, JSON_Object* pArgs)
	{
		if (!canMutate())
		{
			return makeErrorReply(iId, "multiplayer_read_only", "commands are disabled in multiplayer");
		}

		if (strcmp(szName, "set_game_turn") == 0)
		{
			return handleSetGameTurn(iId, pArgs);
		}
		if (strcmp(szName, "set_game_max_turns") == 0)
		{
			return handleSetGameMaxTurns(iId, pArgs);
		}
		if (strcmp(szName, "change_game_max_turns") == 0)
		{
			return handleChangeGameMaxTurns(iId, pArgs);
		}
		if (strcmp(szName, "set_game_start_turn") == 0)
		{
			return handleSetGameStartTurn(iId, pArgs);
		}
		if (strcmp(szName, "set_game_start_year") == 0)
		{
			return handleSetGameStartYear(iId, pArgs);
		}
		if (strcmp(szName, "set_game_estimate_end_turn") == 0)
		{
			return handleSetGameEstimateEndTurn(iId, pArgs);
		}
		if (strcmp(szName, "set_game_target_score") == 0)
		{
			return handleSetGameTargetScore(iId, pArgs);
		}
		if (strcmp(szName, "set_game_max_city_elimination") == 0)
		{
			return handleSetGameMaxCityElimination(iId, pArgs);
		}
		if (strcmp(szName, "set_game_advanced_start_points") == 0)
		{
			return handleSetGameAdvancedStartPoints(iId, pArgs);
		}
		if (strcmp(szName, "set_game_ai_auto_play") == 0)
		{
			return handleSetGameAIAutoPlay(iId, pArgs);
		}
		if (strcmp(szName, "change_game_ai_auto_play") == 0)
		{
			return handleChangeGameAIAutoPlay(iId, pArgs);
		}
		if (strcmp(szName, "change_game_nukes_exploded") == 0)
		{
			return handleChangeGameNukesExploded(iId, pArgs);
		}
		if (strcmp(szName, "set_game_pause_player") == 0)
		{
			return handleSetGamePausePlayer(iId, pArgs);
		}
		if (strcmp(szName, "set_game_winner") == 0)
		{
			return handleSetGameWinner(iId, pArgs);
		}
		if (strcmp(szName, "set_game_state") == 0)
		{
			return handleSetGameState(iId, pArgs);
		}
		if (strcmp(szName, "set_game_option") == 0)
		{
			return handleSetGameOption(iId, pArgs);
		}
		if (strcmp(szName, "set_multiplayer_option") == 0)
		{
			return handleSetMultiplayerOption(iId, pArgs);
		}
		if (strcmp(szName, "set_force_control") == 0)
		{
			return handleSetForceControl(iId, pArgs);
		}
		if (strcmp(szName, "set_player_gold") == 0)
		{
			return handleSetPlayerGold(iId, pArgs);
		}
		if (strcmp(szName, "change_player_gold") == 0)
		{
			return handleChangePlayerGold(iId, pArgs);
		}
		if (strcmp(szName, "set_player_alive") == 0)
		{
			return handleSetPlayerAlive(iId, pArgs);
		}
		if (strcmp(szName, "set_player_playable") == 0)
		{
			return handleSetPlayerPlayable(iId, pArgs);
		}
		if (strcmp(szName, "set_player_current_era") == 0)
		{
			return handleSetPlayerCurrentEra(iId, pArgs);
		}
		if (strcmp(szName, "set_player_personality") == 0)
		{
			return handleSetPlayerPersonality(iId, pArgs);
		}
		if (strcmp(szName, "set_player_parent") == 0)
		{
			return handleSetPlayerParent(iId, pArgs);
		}
		if (strcmp(szName, "set_player_advanced_start_points") == 0)
		{
			return handleSetPlayerAdvancedStartPoints(iId, pArgs);
		}
		if (strcmp(szName, "change_player_advanced_start_points") == 0)
		{
			return handleChangePlayerAdvancedStartPoints(iId, pArgs);
		}
		if (strcmp(szName, "change_player_golden_age_turns") == 0)
		{
			return handleChangePlayerGoldenAgeTurns(iId, pArgs);
		}
		if (strcmp(szName, "change_player_num_unit_golden_ages") == 0)
		{
			return handleChangePlayerNumUnitGoldenAges(iId, pArgs);
		}
		if (strcmp(szName, "change_player_anarchy_turns") == 0)
		{
			return handleChangePlayerAnarchyTurns(iId, pArgs);
		}
		if (strcmp(szName, "change_player_strike_turns") == 0)
		{
			return handleChangePlayerStrikeTurns(iId, pArgs);
		}
		if (strcmp(szName, "set_player_combat_experience") == 0)
		{
			return handleSetPlayerCombatExperience(iId, pArgs);
		}
		if (strcmp(szName, "change_player_combat_experience") == 0)
		{
			return handleChangePlayerCombatExperience(iId, pArgs);
		}
		if (strcmp(szName, "set_player_commerce_percent") == 0)
		{
			return handleSetPlayerCommercePercent(iId, pArgs);
		}
		if (strcmp(szName, "change_player_commerce_percent") == 0)
		{
			return handleChangePlayerCommercePercent(iId, pArgs);
		}
		if (strcmp(szName, "change_player_commerce_rate_modifier") == 0)
		{
			return handleChangePlayerCommerceRateModifier(iId, pArgs);
		}
		if (strcmp(szName, "set_player_gold_per_turn_by_player") == 0)
		{
			return handleSetPlayerGoldPerTurnByPlayer(iId, pArgs);
		}
		if (strcmp(szName, "change_player_gold_per_turn_by_player") == 0)
		{
			return handleChangePlayerGoldPerTurnByPlayer(iId, pArgs);
		}
		if (strcmp(szName, "set_city_population") == 0)
		{
			return handleSetCityPopulation(iId, pArgs);
		}
		if (strcmp(szName, "change_city_population") == 0)
		{
			return handleChangeCityPopulation(iId, pArgs);
		}
		if (strcmp(szName, "set_city_culture") == 0)
		{
			return handleSetCityCulture(iId, pArgs);
		}
		if (strcmp(szName, "set_city_production") == 0)
		{
			return handleSetCityProduction(iId, pArgs);
		}
		if (strcmp(szName, "change_city_production") == 0)
		{
			return handleChangeCityProduction(iId, pArgs);
		}
		if (strcmp(szName, "set_city_unit_production") == 0)
		{
			return handleSetCityUnitProduction(iId, pArgs);
		}
		if (strcmp(szName, "set_city_building_production") == 0)
		{
			return handleSetCityBuildingProduction(iId, pArgs);
		}
		if (strcmp(szName, "set_city_project_production") == 0)
		{
			return handleSetCityProjectProduction(iId, pArgs);
		}
		if (strcmp(szName, "push_city_order") == 0)
		{
			return handlePushCityOrder(iId, pArgs);
		}
		if (strcmp(szName, "clear_city_order_queue") == 0)
		{
			return handleClearCityOrderQueue(iId, pArgs);
		}
		if (strcmp(szName, "pop_city_order") == 0)
		{
			return handlePopCityOrder(iId, pArgs);
		}
		if (strcmp(szName, "set_city_occupation_timer") == 0)
		{
			return handleSetCityOccupationTimer(iId, pArgs);
		}
		if (strcmp(szName, "change_city_occupation_timer") == 0)
		{
			return handleChangeCityOccupationTimer(iId, pArgs);
		}
		if (strcmp(szName, "change_city_hurry_anger_timer") == 0)
		{
			return handleChangeCityHurryAngerTimer(iId, pArgs);
		}
		if (strcmp(szName, "set_city_real_building") == 0)
		{
			return handleSetCityRealBuilding(iId, pArgs);
		}
		if (strcmp(szName, "set_city_free_building") == 0)
		{
			return handleSetCityFreeBuilding(iId, pArgs);
		}
		if (strcmp(szName, "set_city_religion") == 0)
		{
			return handleSetCityReligion(iId, pArgs);
		}
		if (strcmp(szName, "set_city_corporation") == 0)
		{
			return handleSetCityCorporation(iId, pArgs);
		}
		if (strcmp(szName, "set_city_building_happiness_change") == 0)
		{
			return handleSetCityBuildingHappinessChange(iId, pArgs);
		}
		if (strcmp(szName, "set_city_building_health_change") == 0)
		{
			return handleSetCityBuildingHealthChange(iId, pArgs);
		}
		if (strcmp(szName, "set_plot_owner") == 0)
		{
			return handleSetPlotOwner(iId, pArgs);
		}
		if (strcmp(szName, "set_plot_terrain") == 0)
		{
			return handleSetPlotTerrain(iId, pArgs);
		}
		if (strcmp(szName, "set_plot_feature") == 0)
		{
			return handleSetPlotFeature(iId, pArgs);
		}
		if (strcmp(szName, "set_plot_bonus") == 0)
		{
			return handleSetPlotBonus(iId, pArgs);
		}
		if (strcmp(szName, "set_plot_improvement") == 0)
		{
			return handleSetPlotImprovement(iId, pArgs);
		}
		if (strcmp(szName, "set_plot_route") == 0)
		{
			return handleSetPlotRoute(iId, pArgs);
		}
		if (strcmp(szName, "set_plot_culture") == 0)
		{
			return handleSetPlotCulture(iId, pArgs);
		}
		if (strcmp(szName, "change_plot_culture") == 0)
		{
			return handleChangePlotCulture(iId, pArgs);
		}
		if (strcmp(szName, "set_plot_revealed") == 0)
		{
			return handleSetPlotRevealed(iId, pArgs);
		}
		if (strcmp(szName, "set_unit_damage") == 0)
		{
			return handleSetUnitDamage(iId, pArgs);
		}
		if (strcmp(szName, "change_unit_damage") == 0)
		{
			return handleChangeUnitDamage(iId, pArgs);
		}
		if (strcmp(szName, "set_unit_experience") == 0)
		{
			return handleSetUnitExperience(iId, pArgs);
		}
		if (strcmp(szName, "change_unit_experience") == 0)
		{
			return handleChangeUnitExperience(iId, pArgs);
		}
		if (strcmp(szName, "set_unit_xy") == 0)
		{
			return handleSetUnitXY(iId, pArgs);
		}
		if (strcmp(szName, "set_unit_moves") == 0)
		{
			return handleSetUnitMoves(iId, pArgs);
		}
		if (strcmp(szName, "change_unit_moves") == 0)
		{
			return handleChangeUnitMoves(iId, pArgs);
		}
		if (strcmp(szName, "finish_unit_moves") == 0)
		{
			return handleFinishUnitMoves(iId, pArgs);
		}
		if (strcmp(szName, "set_unit_level") == 0)
		{
			return handleSetUnitLevel(iId, pArgs);
		}
		if (strcmp(szName, "change_unit_level") == 0)
		{
			return handleChangeUnitLevel(iId, pArgs);
		}
		if (strcmp(szName, "set_unit_fortify_turns") == 0)
		{
			return handleSetUnitFortifyTurns(iId, pArgs);
		}
		if (strcmp(szName, "change_unit_fortify_turns") == 0)
		{
			return handleChangeUnitFortifyTurns(iId, pArgs);
		}
		if (strcmp(szName, "set_unit_made_attack") == 0)
		{
			return handleSetUnitMadeAttack(iId, pArgs);
		}
		if (strcmp(szName, "set_unit_base_combat") == 0)
		{
			return handleSetUnitBaseCombat(iId, pArgs);
		}
		if (strcmp(szName, "set_unit_immobile_timer") == 0)
		{
			return handleSetUnitImmobileTimer(iId, pArgs);
		}
		if (strcmp(szName, "change_unit_immobile_timer") == 0)
		{
			return handleChangeUnitImmobileTimer(iId, pArgs);
		}
		if (strcmp(szName, "set_unit_promotion") == 0)
		{
			return handleSetUnitPromotion(iId, pArgs);
		}
		if (strcmp(szName, "kill_unit") == 0)
		{
			return handleKillUnit(iId, pArgs);
		}
		if (strcmp(szName, "spawn_unit") == 0)
		{
			return handleSpawnUnit(iId, pArgs);
		}
		if (strcmp(szName, "set_mod_state") == 0)
		{
			return handleSetModState(iId, pArgs);
		}
		if (strcmp(szName, "set_player_civic") == 0)
		{
			return handleSetPlayerCivic(iId, pArgs);
		}
		if (strcmp(szName, "set_player_state_religion") == 0)
		{
			return handleSetPlayerStateReligion(iId, pArgs);
		}
		if (strcmp(szName, "set_player_research") == 0)
		{
			return handleSetPlayerResearch(iId, pArgs);
		}
		if (strcmp(szName, "set_team_has_tech") == 0)
		{
			return handleSetTeamHasTech(iId, pArgs);
		}
		if (strcmp(szName, "change_team_research_progress") == 0)
		{
			return handleChangeTeamResearchProgress(iId, pArgs);
		}
		if (strcmp(szName, "meet_team") == 0)
		{
			return handleMeetTeam(iId, pArgs);
		}
		if (strcmp(szName, "declare_war") == 0)
		{
			return handleDeclareWar(iId, pArgs);
		}
		if (strcmp(szName, "make_peace") == 0)
		{
			return handleMakePeace(iId, pArgs);
		}
		if (strcmp(szName, "set_team_open_borders") == 0)
		{
			return handleSetTeamOpenBorders(iId, pArgs);
		}
		if (strcmp(szName, "set_team_defensive_pact") == 0)
		{
			return handleSetTeamDefensivePact(iId, pArgs);
		}
		if (strcmp(szName, "set_team_force_peace") == 0)
		{
			return handleSetTeamForcePeace(iId, pArgs);
		}
		if (strcmp(szName, "set_team_permanent_war_peace") == 0)
		{
			return handleSetTeamPermanentWarPeace(iId, pArgs);
		}
		if (strcmp(szName, "set_team_vassal") == 0)
		{
			return handleSetTeamVassal(iId, pArgs);
		}
		if (strcmp(szName, "set_team_war_weariness") == 0)
		{
			return handleSetTeamWarWeariness(iId, pArgs);
		}
		if (strcmp(szName, "change_team_war_weariness") == 0)
		{
			return handleChangeTeamWarWeariness(iId, pArgs);
		}
		if (strcmp(szName, "set_team_stolen_visibility_timer") == 0)
		{
			return handleSetTeamStolenVisibilityTimer(iId, pArgs);
		}
		if (strcmp(szName, "change_team_stolen_visibility_timer") == 0)
		{
			return handleChangeTeamStolenVisibilityTimer(iId, pArgs);
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
					json_array_append_string(pCaps, "callback_requests");
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

	bool parseBoolFieldReply(const CvString& szLine, int iId, const char* szField, bool bAllowMissing, bool& bValue)
	{
		JSON_Value* pValue = json_parse_string(szLine.GetCString());
		if (pValue == NULL || json_value_get_type(pValue) != JSONObject)
		{
			json_value_free(pValue);
			return false;
		}

		JSON_Object* pObject = json_value_get_object(pValue);
		const char* szType = json_object_get_string(pObject, "type");
		int iReplyId = (int)json_object_get_number(pObject, "id");
		if (szType == NULL || strcmp(szType, "reply") != 0 || iReplyId != iId)
		{
			json_value_free(pValue);
			return false;
		}

		bool bHandled = false;
		if (json_object_get_boolean(pObject, "ok"))
		{
			JSON_Object* pResult = json_object_get_object(pObject, "result");
			if (pResult != NULL)
			{
				JSON_Value* pField = json_object_get_value(pResult, szField);
				if (pField != NULL)
				{
					if (json_value_get_type(pField) == JSONBoolean)
					{
						bValue = json_value_get_boolean(pField) ? true : false;
						bHandled = true;
					}
					else if (json_value_get_type(pField) == JSONNumber)
					{
						bValue = ((int)json_value_get_number(pField)) != 0;
						bHandled = true;
					}
				}
			}
		}

		json_value_free(pValue);
		return bHandled || bAllowMissing;
	}

	bool parseConsumeReply(const CvString& szLine, int iId, bool& bConsumed)
	{
		return parseBoolFieldReply(szLine, iId, "consume", true, bConsumed);
	}

	bool parseValueReply(const CvString& szLine, int iId, bool& bValue)
	{
		return parseBoolFieldReply(szLine, iId, "value", false, bValue);
	}

	bool sendCallbackRequest(int iId, const char* szName, const char* szArgsJson)
	{
		if (!g_bEnabled || !g_kCallbackPipe.bConnected || szName == NULL)
		{
			return false;
		}

		JSON_Value* pValue = makeBaseMessage("callback_request");
		JSON_Object* pObject = json_value_get_object(pValue);
		json_object_set_number(pObject, "id", iId);
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

		return writeLine(g_kCallbackPipe, serializeAndFree(pValue));
	}

	bool waitForConsumeReply(int iId, bool& bConsumed)
	{
		DWORD dwStarted = GetTickCount();
		DWORD dwTimeout = getCallbackTimeoutMs();

		for (;;)
		{
			readAvailable(g_kCallbackPipe);

			CvString szLine;
			while (popBufferedLine(g_kCallbackPipe, szLine))
			{
				if (!szLine.empty() && parseConsumeReply(szLine, iId, bConsumed))
				{
					return true;
				}
			}

			pollPipe(g_kControlPipe, true);

			if (!g_kCallbackPipe.bConnected || GetTickCount() - dwStarted >= dwTimeout)
			{
				return false;
			}

			Sleep(1);
		}
	}

	bool waitForValueReply(int iId, bool& bValue)
	{
		DWORD dwStarted = GetTickCount();
		DWORD dwTimeout = getCallbackTimeoutMs();

		for (;;)
		{
			readAvailable(g_kCallbackPipe);

			CvString szLine;
			while (popBufferedLine(g_kCallbackPipe, szLine))
			{
				if (!szLine.empty() && parseValueReply(szLine, iId, bValue))
				{
					return true;
				}
			}

			pollPipe(g_kControlPipe, true);

			if (!g_kCallbackPipe.bConnected || GetTickCount() - dwStarted >= dwTimeout)
			{
				return false;
			}

			Sleep(1);
		}
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
	startCompanionProcess();
	OutputDebugString("CvGameBridge: enabled\n");
}

void CvGameBridge::shutdown()
{
	stopCompanionProcess();
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

bool CvGameBridge::requestCallbackConsume(const char* szName, const char* szArgsJson, bool& bConsumed)
{
	if (!g_bEnabled || !g_kCallbackPipe.bConnected)
	{
		return false;
	}

	int iId = (int)g_uiNextCallbackId++;
	if (!sendCallbackRequest(iId, szName, szArgsJson))
	{
		return false;
	}

	if (!waitForConsumeReply(iId, bConsumed))
	{
		return false;
	}

	return true;
}

bool CvGameBridge::requestCallbackBool(const char* szName, const char* szArgsJson, bool& bValue)
{
	if (!g_bEnabled || !g_kCallbackPipe.bConnected)
	{
		return false;
	}

	int iId = (int)g_uiNextCallbackId++;
	if (!sendCallbackRequest(iId, szName, szArgsJson))
	{
		return false;
	}

	if (!waitForValueReply(iId, bValue))
	{
		return false;
	}

	return true;
}
