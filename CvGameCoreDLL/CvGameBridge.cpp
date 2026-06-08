#include "CvGameCoreDLL.h"
#include "CvGameBridge.h"

#include "CvDLLInterfaceIFaceBase.h"
#include "CvDLLEngineIFaceBase.h"
#include "ThirdParty/parson/parson.h"

namespace
{
	static const DWORD PIPE_BUFFER_SIZE = 8192;
	static const DWORD DEFAULT_CALLBACK_TIMEOUT_MS = 50;
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

	void closePipe(BridgePipe& kPipe);
	void pollPipe(BridgePipe& kPipe, bool bControl);

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

	bool validPlayer(int iPlayer)
	{
		return (iPlayer >= 0 && iPlayer < GC.getMAX_PLAYERS());
	}

	bool validTeam(int iTeam)
	{
		return (iTeam >= 0 && iTeam < MAX_TEAMS);
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

	void setPlayerState(JSON_Object* pResult, int iPlayer)
	{
		CvPlayer& kPlayer = GET_PLAYER((PlayerTypes)iPlayer);
		json_object_set_number(pResult, "player", iPlayer);
		json_object_set_number(pResult, "team", kPlayer.getTeam());
		json_object_set_boolean(pResult, "alive", kPlayer.isAlive() ? 1 : 0);
		json_object_set_boolean(pResult, "human", kPlayer.isHuman() ? 1 : 0);
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

		if (strcmp(szName, "get_player_state") == 0)
		{
			int iPlayer = -1;
			if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
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
			if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
			{
				return makeErrorReply(iId, "bad_player", "player is missing or out of range");
			}
			return makePlayerOptionsReply(iId, iPlayer);
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
		if (!getInt(pArgs, "player", iPlayer) || !validPlayer(iPlayer))
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
		if (strcmp(szName, "change_player_gold") == 0)
		{
			return handleChangePlayerGold(iId, pArgs);
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

	bool parseConsumeReply(const CvString& szLine, int iId, bool& bConsumed)
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

		bool bHandled = true;
		if (json_object_get_boolean(pObject, "ok"))
		{
			JSON_Object* pResult = json_object_get_object(pObject, "result");
			if (pResult != NULL)
			{
				JSON_Value* pConsume = json_object_get_value(pResult, "consume");
				if (pConsume != NULL)
				{
					if (json_value_get_type(pConsume) == JSONBoolean)
					{
						bConsumed = json_value_get_boolean(pConsume) ? true : false;
					}
					else if (json_value_get_type(pConsume) == JSONNumber)
					{
						bConsumed = ((int)json_value_get_number(pConsume)) != 0;
					}
				}
			}
		}

		json_value_free(pValue);
		return bHandled;
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
