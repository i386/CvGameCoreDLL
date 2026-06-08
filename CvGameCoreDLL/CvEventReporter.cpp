#include "CvGameCoreDLL.h"
#include "CvEventReporter.h"
#include "CvDllPythonEvents.h"
#include "CvDLLEngineIFaceBase.h"
#include "CvInitCore.h"
#include "CvGameBridge.h"

namespace
{
	static const unsigned int PRESAVE_CALLBACK_TIMEOUT_MS = 5000;

	void bridgeSignal(const char* szName)
	{
		CvGameBridge::sendEvent(szName);
		CvGameBridge::sendCallbackMirror(szName);
	}

	void bridgePayload(const char* szName, const CvString& szArgs)
	{
		CvGameBridge::sendEvent(szName, szArgs.GetCString());
		CvGameBridge::sendCallbackMirror(szName, szArgs.GetCString());
	}

	CvString jsonEscape(const CvString& szValue)
	{
		CvString szEscaped;
		for (int iI = 0; iI < (int)szValue.length(); ++iI)
		{
			unsigned char ch = (unsigned char)szValue[iI];
			switch (ch)
			{
			case '"':
				szEscaped += "\\\"";
				break;
			case '\\':
				szEscaped += "\\\\";
				break;
			case '\b':
				szEscaped += "\\b";
				break;
			case '\f':
				szEscaped += "\\f";
				break;
			case '\n':
				szEscaped += "\\n";
				break;
			case '\r':
				szEscaped += "\\r";
				break;
			case '\t':
				szEscaped += "\\t";
				break;
			default:
				if (ch < 32)
				{
					char szHex[8];
					sprintf(szHex, "\\u%04x", ch);
					szEscaped += szHex;
				}
				else
				{
					szEscaped += (char)ch;
				}
				break;
			}
		}
		return szEscaped;
	}
}

//
// static, singleton accessor
//
CvEventReporter& CvEventReporter::getInstance()
{
	static CvEventReporter gEventReporter;
	return gEventReporter;
}


void CvEventReporter::resetStatistics()
{
	m_kStatistics.reset();
}

//
// Returns true if the event is consumed by Python
//
bool CvEventReporter::mouseEvent(int evt, int iCursorX, int iCursorY, bool bInterfaceConsumed)
{
	if (CvGameBridge::isEnabled())
	{
		NiPoint3 pt3Location;
		CvPlot* pPlot = gDLL->getEngineIFace()->pickPlot(iCursorX, iCursorY, pt3Location);
		CvString szArgs;
		szArgs.Format("{\"evt\":%d,\"cursor_x\":%d,\"cursor_y\":%d,\"x\":%d,\"y\":%d,\"interface_consumed\":%d}", evt, iCursorX, iCursorY, pPlot ? pPlot->getX() : -1, pPlot ? pPlot->getY() : -1, bInterfaceConsumed ? 1 : 0);
		bool bConsumed = false;
		if (CvGameBridge::requestCallbackConsume("mouse_event", szArgs.GetCString(), bConsumed))
		{
			return bConsumed;
		}
	}
	return m_kPythonEventMgr.reportMouseEvent(evt, iCursorX, iCursorY, bInterfaceConsumed);
}

//
// Returns true if the event is consumed by Python
//
bool CvEventReporter::kbdEvent(int evt, int key, int iCursorX, int iCursorY)
{
	if (CvGameBridge::isEnabled())
	{
		NiPoint3 pt3Location;
		CvPlot* pPlot = gDLL->getEngineIFace()->pickPlot(iCursorX, iCursorY, pt3Location);
		CvString szArgs;
		szArgs.Format("{\"evt\":%d,\"key\":%d,\"cursor_x\":%d,\"cursor_y\":%d,\"x\":%d,\"y\":%d}", evt, key, iCursorX, iCursorY, pPlot ? pPlot->getX() : -1, pPlot ? pPlot->getY() : -1);
		bool bConsumed = false;
		if (CvGameBridge::requestCallbackConsume("kbd_event", szArgs.GetCString(), bConsumed))
		{
			return bConsumed;
		}
	}
	return m_kPythonEventMgr.reportKbdEvent(evt, key, iCursorX, iCursorY);
}

void CvEventReporter::genericEvent(const char* szEventName, void *pyArgs)
{
	m_kPythonEventMgr.reportGenericEvent(szEventName, pyArgs);
}


void CvEventReporter::newGame()
{
	// This will only be called if statistics are being reported!
	// Called at the launch of a game (new or loaded)

	// Report initial stats for the game
	m_kStatistics.setMapName( CvString(GC.getInitCore().getMapScriptName()).GetCString() );
	m_kStatistics.setEra(GC.getInitCore().getEra());
}

void CvEventReporter::newPlayer(PlayerTypes ePlayer)
{
	// This will only be called if statistics are being reported!
	// Called at the launch of a game (new or loaded)

	// Report initial stats for this player
	m_kStatistics.setLeader(ePlayer, GET_PLAYER(ePlayer).getLeaderType());
}

void CvEventReporter::reportModNetMessage(int iData1, int iData2, int iData3, int iData4, int iData5)
{
	CvString szArgs;
	szArgs.Format("{\"data1\":%d,\"data2\":%d,\"data3\":%d,\"data4\":%d,\"data5\":%d}", iData1, iData2, iData3, iData4, iData5);
	bridgePayload("mod_net_message", szArgs);
	m_kPythonEventMgr.reportModNetMessage(iData1, iData2, iData3, iData4, iData5);
}

void CvEventReporter::init()
{
	bridgeSignal("init");
	m_kPythonEventMgr.reportInit();
}

void CvEventReporter::update(float fDeltaTime)
{
	if (GC.getUSE_ON_UPDATE_CALLBACK())
	{
		CvString szArgs;
		szArgs.Format("{\"delta_time\":%f}", fDeltaTime);
		bridgePayload("update", szArgs);
	}
	m_kPythonEventMgr.reportUpdate(fDeltaTime);
}

void CvEventReporter::unInit()
{
	bridgeSignal("uninit");
	m_kPythonEventMgr.reportUnInit();
}

void CvEventReporter::gameStart()
{
	bridgeSignal("game_start");
	m_kPythonEventMgr.reportGameStart();
}

void CvEventReporter::gameEnd()
{
	bridgeSignal("game_end");
	m_kPythonEventMgr.reportGameEnd();
}

void CvEventReporter::beginGameTurn(int iGameTurn)
{
	CvString szArgs;
	szArgs.Format("{\"turn\":%d}", iGameTurn);
	bridgePayload("begin_game_turn", szArgs);
	m_kPythonEventMgr.reportBeginGameTurn(iGameTurn);
}

void CvEventReporter::endGameTurn(int iGameTurn)
{
	CvString szArgs;
	szArgs.Format("{\"turn\":%d}", iGameTurn);
	bridgePayload("end_game_turn", szArgs);
	m_kPythonEventMgr.reportEndGameTurn(iGameTurn);
}

void CvEventReporter::beginPlayerTurn(int iGameTurn, PlayerTypes ePlayer)
{
	CvString szArgs;
	szArgs.Format("{\"turn\":%d,\"player\":%d}", iGameTurn, ePlayer);
	bridgePayload("begin_player_turn", szArgs);
	m_kPythonEventMgr.reportBeginPlayerTurn(iGameTurn, ePlayer);
}

void CvEventReporter::endPlayerTurn(int iGameTurn, PlayerTypes ePlayer)
{
	CvString szArgs;
	szArgs.Format("{\"turn\":%d,\"player\":%d}", iGameTurn, ePlayer);
	bridgePayload("end_player_turn", szArgs);
	m_kPythonEventMgr.reportEndPlayerTurn(iGameTurn, ePlayer);
}

void CvEventReporter::firstContact(TeamTypes eTeamID1, TeamTypes eTeamID2)
{
	CvString szArgs;
	szArgs.Format("{\"team\":%d,\"other_team\":%d}", eTeamID1, eTeamID2);
	bridgePayload("first_contact", szArgs);
	m_kPythonEventMgr.reportFirstContact(eTeamID1, eTeamID2);
}

void CvEventReporter::combatResult(CvUnit* pWinner, CvUnit* pLoser)
{
	if (pWinner != NULL && pLoser != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"winner_player\":%d,\"winner_unit\":%d,\"winner_unit_type\":%d,\"winner_x\":%d,\"winner_y\":%d,\"loser_player\":%d,\"loser_unit\":%d,\"loser_unit_type\":%d,\"loser_x\":%d,\"loser_y\":%d}",
			pWinner->getOwnerINLINE(), pWinner->getID(), pWinner->getUnitType(), pWinner->getX_INLINE(), pWinner->getY_INLINE(),
			pLoser->getOwnerINLINE(), pLoser->getID(), pLoser->getUnitType(), pLoser->getX_INLINE(), pLoser->getY_INLINE());
		bridgePayload("combat_result", szArgs);
	}
	m_kPythonEventMgr.reportCombatResult(pWinner, pLoser);
}

void CvEventReporter::improvementBuilt(int iImprovementType, int iX, int iY)
{
	CvString szArgs;
	szArgs.Format("{\"improvement\":%d,\"x\":%d,\"y\":%d}", iImprovementType, iX, iY);
	bridgePayload("improvement_built", szArgs);
	m_kPythonEventMgr.reportImprovementBuilt(iImprovementType, iX, iY);
}

void CvEventReporter::improvementDestroyed(int iImprovementType, int iPlayer, int iX, int iY)
{
	CvString szArgs;
	szArgs.Format("{\"improvement\":%d,\"player\":%d,\"x\":%d,\"y\":%d}", iImprovementType, iPlayer, iX, iY);
	bridgePayload("improvement_destroyed", szArgs);
	m_kPythonEventMgr.reportImprovementDestroyed(iImprovementType, iPlayer, iX, iY);
}

void CvEventReporter::routeBuilt(int iRouteType, int iX, int iY)
{
	CvString szArgs;
	szArgs.Format("{\"route\":%d,\"x\":%d,\"y\":%d}", iRouteType, iX, iY);
	bridgePayload("route_built", szArgs);
	m_kPythonEventMgr.reportRouteBuilt(iRouteType, iX, iY);
}

void CvEventReporter::plotRevealed(CvPlot *pPlot, TeamTypes eTeam)
{
	if (pPlot != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"x\":%d,\"y\":%d,\"team\":%d}", pPlot->getX_INLINE(), pPlot->getY_INLINE(), eTeam);
		bridgePayload("plot_revealed", szArgs);
	}
	m_kPythonEventMgr.reportPlotRevealed(pPlot, eTeam);
}

void CvEventReporter::plotFeatureRemoved(CvPlot *pPlot, FeatureTypes eFeature, CvCity* pCity)
{
	if (pPlot != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"x\":%d,\"y\":%d,\"feature\":%d,\"city_player\":%d,\"city\":%d}",
			pPlot->getX_INLINE(), pPlot->getY_INLINE(), eFeature,
			pCity ? pCity->getOwnerINLINE() : -1, pCity ? pCity->getID() : -1);
		bridgePayload("plot_feature_removed", szArgs);
	}
	m_kPythonEventMgr.reportPlotFeatureRemoved(pPlot, eFeature, pCity);
}

void CvEventReporter::plotPicked(CvPlot *pPlot)
{
	if (pPlot != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"x\":%d,\"y\":%d}", pPlot->getX_INLINE(), pPlot->getY_INLINE());
		bridgePayload("plot_picked", szArgs);
	}
	m_kPythonEventMgr.reportPlotPicked(pPlot);
}

void CvEventReporter::nukeExplosion(CvPlot *pPlot, CvUnit* pNukeUnit)
{
	if (pPlot != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"x\":%d,\"y\":%d,\"player\":%d,\"unit\":%d,\"unit_type\":%d}",
			pPlot->getX_INLINE(), pPlot->getY_INLINE(),
			pNukeUnit ? pNukeUnit->getOwnerINLINE() : -1,
			pNukeUnit ? pNukeUnit->getID() : -1,
			pNukeUnit ? pNukeUnit->getUnitType() : -1);
		bridgePayload("nuke_explosion", szArgs);
	}
	m_kPythonEventMgr.reportNukeExplosion(pPlot, pNukeUnit);
}

void CvEventReporter::gotoPlotSet(CvPlot *pPlot, PlayerTypes ePlayer)
{
	if (pPlot != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"x\":%d,\"y\":%d,\"player\":%d}", pPlot->getX_INLINE(), pPlot->getY_INLINE(), ePlayer);
		bridgePayload("goto_plot_set", szArgs);
	}
	m_kPythonEventMgr.reportGotoPlotSet(pPlot, ePlayer);
}

void CvEventReporter::cityBuilt( CvCity *pCity )
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"x\":%d,\"y\":%d}", pCity->getOwnerINLINE(), pCity->getID(), pCity->getX_INLINE(), pCity->getY_INLINE());
		bridgePayload("city_built", szArgs);
	}
	m_kPythonEventMgr.reportCityBuilt(pCity);
	m_kStatistics.cityBuilt(pCity);
}

void CvEventReporter::cityRazed( CvCity *pCity, PlayerTypes ePlayer )
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"razed_by\":%d,\"x\":%d,\"y\":%d}", pCity->getOwnerINLINE(), pCity->getID(), ePlayer, pCity->getX_INLINE(), pCity->getY_INLINE());
		bridgePayload("city_razed", szArgs);
	}
	m_kPythonEventMgr.reportCityRazed(pCity, ePlayer);
	m_kStatistics.cityRazed(pCity, ePlayer);
}

void CvEventReporter::cityAcquired(PlayerTypes eOldOwner, PlayerTypes iPlayer, CvCity* pCity, bool bConquest, bool bTrade)
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"old_player\":%d,\"player\":%d,\"city\":%d,\"conquest\":%d,\"trade\":%d,\"x\":%d,\"y\":%d}", eOldOwner, iPlayer, pCity->getID(), bConquest ? 1 : 0, bTrade ? 1 : 0, pCity->getX_INLINE(), pCity->getY_INLINE());
		bridgePayload("city_acquired", szArgs);
	}
	m_kPythonEventMgr.reportCityAcquired(eOldOwner, iPlayer, pCity, bConquest, bTrade);
}

void CvEventReporter::cityAcquiredAndKept(PlayerTypes iPlayer, CvCity* pCity)
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"x\":%d,\"y\":%d}", iPlayer, pCity->getID(), pCity->getX_INLINE(), pCity->getY_INLINE());
		bridgePayload("city_acquired_kept", szArgs);
	}
	m_kPythonEventMgr.reportCityAcquiredAndKept(iPlayer, pCity);
}

void CvEventReporter::cityLost( CvCity *pCity)
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"x\":%d,\"y\":%d}", pCity->getOwnerINLINE(), pCity->getID(), pCity->getX_INLINE(), pCity->getY_INLINE());
		bridgePayload("city_lost", szArgs);
	}
	m_kPythonEventMgr.reportCityLost(pCity);
}

void CvEventReporter::cultureExpansion( CvCity *pCity, PlayerTypes ePlayer )
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"x\":%d,\"y\":%d}", ePlayer, pCity->getID(), pCity->getX_INLINE(), pCity->getY_INLINE());
		bridgePayload("culture_expansion", szArgs);
	}
	m_kPythonEventMgr.reportCultureExpansion(pCity, ePlayer);
}

void CvEventReporter::cityGrowth(CvCity *pCity, PlayerTypes ePlayer)
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"population\":%d}", ePlayer, pCity->getID(), pCity->getPopulation());
		bridgePayload("city_growth", szArgs);
	}
	m_kPythonEventMgr.reportCityGrowth(pCity, ePlayer);
}

void CvEventReporter::cityDoTurn( CvCity *pCity, PlayerTypes ePlayer )
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"x\":%d,\"y\":%d}", ePlayer, pCity->getID(), pCity->getX_INLINE(), pCity->getY_INLINE());
		bridgePayload("city_do_turn", szArgs);
	}
	m_kPythonEventMgr.reportCityProduction(pCity, ePlayer);
}

void CvEventReporter::cityBuildingUnit(CvCity* pCity, UnitTypes eUnitType)
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"unit_type\":%d}", pCity->getOwnerINLINE(), pCity->getID(), eUnitType);
		bridgePayload("city_building_unit", szArgs);
	}
	m_kPythonEventMgr.reportCityBuildingUnit(pCity, eUnitType);
}

void CvEventReporter::cityBuildingBuilding(CvCity* pCity, BuildingTypes eBuildingType)
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"building\":%d}", pCity->getOwnerINLINE(), pCity->getID(), eBuildingType);
		bridgePayload("city_building_building", szArgs);
	}
	m_kPythonEventMgr.reportCityBuildingBuilding(pCity, eBuildingType);
}

void CvEventReporter::cityRename(CvCity* pCity)
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"x\":%d,\"y\":%d}", pCity->getOwnerINLINE(), pCity->getID(), pCity->getX_INLINE(), pCity->getY_INLINE());
		bridgePayload("city_rename", szArgs);
	}
	m_kPythonEventMgr.reportCityRename(pCity);
}

void CvEventReporter::cityHurry(CvCity* pCity, HurryTypes eHurry)
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"hurry\":%d}", pCity->getOwnerINLINE(), pCity->getID(), eHurry);
		bridgePayload("city_hurry", szArgs);
	}
	m_kPythonEventMgr.reportCityHurry(pCity, eHurry);
}

void CvEventReporter::selectionGroupPushMission(CvSelectionGroup* pSelectionGroup, MissionTypes eMission)
{
	if (pSelectionGroup != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"group\":%d,\"mission\":%d}", pSelectionGroup->getOwnerINLINE(), pSelectionGroup->getID(), eMission);
		bridgePayload("selection_group_push_mission", szArgs);
	}
	m_kPythonEventMgr.reportSelectionGroupPushMission(pSelectionGroup, eMission);
}

void CvEventReporter::unitMove(CvPlot* pPlot, CvUnit* pUnit, CvPlot* pOldPlot)
{
	if (pPlot != NULL && pUnit != NULL && pOldPlot != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"from_x\":%d,\"from_y\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pOldPlot->getX_INLINE(), pOldPlot->getY_INLINE(), pPlot->getX_INLINE(), pPlot->getY_INLINE());
		bridgePayload("unit_move", szArgs);
	}
	m_kPythonEventMgr.reportUnitMove(pPlot, pUnit, pOldPlot);
}

void CvEventReporter::unitSetXY(CvPlot* pPlot, CvUnit* pUnit)
{
	if (pPlot != NULL && pUnit != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"unit_type\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pUnit->getUnitType(), pPlot->getX_INLINE(), pPlot->getY_INLINE());
		bridgePayload("unit_set_xy", szArgs);
	}
	m_kPythonEventMgr.reportUnitSetXY(pPlot, pUnit);
}

void CvEventReporter::unitCreated(CvUnit *pUnit)
{
	if (pUnit != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"unit_type\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pUnit->getUnitType(), pUnit->getX_INLINE(), pUnit->getY_INLINE());
		bridgePayload("unit_created", szArgs);
	}
	m_kPythonEventMgr.reportUnitCreated(pUnit);
}

void CvEventReporter::unitBuilt(CvCity *pCity, CvUnit *pUnit)
{
	if (pCity != NULL && pUnit != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"unit\":%d,\"unit_type\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pCity->getID(), pUnit->getID(), pUnit->getUnitType(), pUnit->getX_INLINE(), pUnit->getY_INLINE());
		bridgePayload("unit_built", szArgs);
	}
	m_kPythonEventMgr.reportUnitBuilt(pCity, pUnit);
	m_kStatistics.unitBuilt(pUnit);
}

void CvEventReporter::unitKilled(CvUnit *pUnit, PlayerTypes eAttacker )
{
	if (pUnit != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"unit_type\":%d,\"attacker\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pUnit->getUnitType(), eAttacker, pUnit->getX_INLINE(), pUnit->getY_INLINE());
		bridgePayload("unit_killed", szArgs);
	}
	m_kPythonEventMgr.reportUnitKilled(pUnit, eAttacker);
	m_kStatistics.unitKilled(pUnit, eAttacker);
}

void CvEventReporter::unitLost(CvUnit *pUnit)
{
	if (pUnit != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"unit_type\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pUnit->getUnitType(), pUnit->getX_INLINE(), pUnit->getY_INLINE());
		bridgePayload("unit_lost", szArgs);
	}
	m_kPythonEventMgr.reportUnitLost(pUnit);
}

void CvEventReporter::unitPromoted(CvUnit *pUnit, PromotionTypes ePromotion)
{
	if (pUnit != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"unit_type\":%d,\"promotion\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pUnit->getUnitType(), ePromotion, pUnit->getX_INLINE(), pUnit->getY_INLINE());
		bridgePayload("unit_promoted", szArgs);
	}
	m_kPythonEventMgr.reportUnitPromoted(pUnit, ePromotion);
}

void CvEventReporter::unitSelected( CvUnit *pUnit)
{
	if (pUnit != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"unit_type\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pUnit->getUnitType(), pUnit->getX_INLINE(), pUnit->getY_INLINE());
		bridgePayload("unit_selected", szArgs);
	}
	m_kPythonEventMgr.reportUnitSelected(pUnit);
}

void CvEventReporter::unitRename(CvUnit* pUnit)
{
	if (pUnit != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"unit_type\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pUnit->getUnitType(), pUnit->getX_INLINE(), pUnit->getY_INLINE());
		bridgePayload("unit_rename", szArgs);
	}
	m_kPythonEventMgr.reportUnitRename(pUnit);
}

void CvEventReporter::unitPillage(CvUnit* pUnit, ImprovementTypes eImprovement, RouteTypes eRoute, PlayerTypes ePlayer)
{
	if (pUnit != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"unit_type\":%d,\"improvement\":%d,\"route\":%d,\"pillage_player\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pUnit->getUnitType(), eImprovement, eRoute, ePlayer, pUnit->getX_INLINE(), pUnit->getY_INLINE());
		bridgePayload("unit_pillage", szArgs);
	}
	m_kPythonEventMgr.reportUnitPillage(pUnit, eImprovement, eRoute, ePlayer);
}

void CvEventReporter::unitSpreadReligionAttempt(CvUnit* pUnit, ReligionTypes eReligion, bool bSuccess)
{
	if (pUnit != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"unit_type\":%d,\"religion\":%d,\"success\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pUnit->getUnitType(), eReligion, bSuccess ? 1 : 0, pUnit->getX_INLINE(), pUnit->getY_INLINE());
		bridgePayload("unit_spread_religion_attempt", szArgs);
	}
	m_kPythonEventMgr.reportUnitSpreadReligionAttempt(pUnit, eReligion, bSuccess);
}

void CvEventReporter::unitGifted(CvUnit* pUnit, PlayerTypes eGiftingPlayer, CvPlot* pPlotLocation)
{
	if (pUnit != NULL && pPlotLocation != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"unit_type\":%d,\"gifting_player\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pUnit->getUnitType(), eGiftingPlayer, pPlotLocation->getX_INLINE(), pPlotLocation->getY_INLINE());
		bridgePayload("unit_gifted", szArgs);
	}
	m_kPythonEventMgr.reportUnitGifted(pUnit, eGiftingPlayer, pPlotLocation);
}

void CvEventReporter::unitBuildImprovement(CvUnit* pUnit, BuildTypes eBuild, bool bFinished)
{
	if (pUnit != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"unit\":%d,\"unit_type\":%d,\"build\":%d,\"finished\":%d,\"x\":%d,\"y\":%d}", pUnit->getOwnerINLINE(), pUnit->getID(), pUnit->getUnitType(), eBuild, bFinished ? 1 : 0, pUnit->getX_INLINE(), pUnit->getY_INLINE());
		bridgePayload("unit_build_improvement", szArgs);
	}
	m_kPythonEventMgr.reportUnitBuildImprovement(pUnit, eBuild, bFinished);
}

void CvEventReporter::goodyReceived(PlayerTypes ePlayer, CvPlot *pGoodyPlot, CvUnit *pGoodyUnit, GoodyTypes eGoodyType)
{
	if (pGoodyPlot != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"x\":%d,\"y\":%d,\"unit_player\":%d,\"unit\":%d,\"unit_type\":%d,\"goody\":%d}",
			ePlayer, pGoodyPlot->getX_INLINE(), pGoodyPlot->getY_INLINE(),
			pGoodyUnit ? pGoodyUnit->getOwnerINLINE() : -1,
			pGoodyUnit ? pGoodyUnit->getID() : -1,
			pGoodyUnit ? pGoodyUnit->getUnitType() : -1,
			eGoodyType);
		bridgePayload("goody_received", szArgs);
	}
	m_kPythonEventMgr.reportGoodyReceived(ePlayer, pGoodyPlot, pGoodyUnit, eGoodyType);
}

void CvEventReporter::greatPersonBorn(CvUnit *pUnit, PlayerTypes ePlayer, CvCity *pCity)
{
	if (pUnit != NULL && pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city_player\":%d,\"city\":%d,\"unit\":%d,\"unit_type\":%d,\"x\":%d,\"y\":%d}",
			ePlayer, pCity->getOwnerINLINE(), pCity->getID(), pUnit->getID(), pUnit->getUnitType(), pUnit->getX_INLINE(), pUnit->getY_INLINE());
		bridgePayload("great_person_born", szArgs);
	}
	m_kPythonEventMgr.reportGreatPersonBorn( pUnit, ePlayer, pCity);
	m_kStatistics.unitBuilt(pUnit);
}

void CvEventReporter::buildingBuilt(CvCity *pCity, BuildingTypes eBuilding)
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"building\":%d}", pCity->getOwnerINLINE(), pCity->getID(), eBuilding);
		bridgePayload("building_built", szArgs);
	}
	m_kPythonEventMgr.reportBuildingBuilt(pCity, eBuilding);
	m_kStatistics.buildingBuilt(pCity, eBuilding);
}

void CvEventReporter::projectBuilt(CvCity *pCity, ProjectTypes eProject)
{
	if (pCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"project\":%d}", pCity->getOwnerINLINE(), pCity->getID(), eProject);
		bridgePayload("project_built", szArgs);
	}
	m_kPythonEventMgr.reportProjectBuilt(pCity, eProject);
}

void CvEventReporter::techAcquired(TechTypes eType, TeamTypes eTeam, PlayerTypes ePlayer, bool bAnnounce)
{
	CvString szArgs;
	szArgs.Format("{\"team\":%d,\"player\":%d,\"tech\":%d,\"announce\":%d}", eTeam, ePlayer, eType, bAnnounce ? 1 : 0);
	bridgePayload("tech_acquired", szArgs);
	m_kPythonEventMgr.reportTechAcquired(eType, eTeam, ePlayer, bAnnounce);
}

void CvEventReporter::techSelected(TechTypes eTech, PlayerTypes ePlayer)
{
	CvString szArgs;
	szArgs.Format("{\"player\":%d,\"tech\":%d}", ePlayer, eTech);
	bridgePayload("tech_selected", szArgs);
	m_kPythonEventMgr.reportTechSelected(eTech, ePlayer);
}

void CvEventReporter::religionFounded(ReligionTypes eType, PlayerTypes ePlayer)
{
	CvString szArgs;
	szArgs.Format("{\"player\":%d,\"religion\":%d}", ePlayer, eType);
	bridgePayload("religion_founded", szArgs);
	m_kPythonEventMgr.reportReligionFounded(eType, ePlayer);
	m_kStatistics.religionFounded(eType, ePlayer);
}

void CvEventReporter::religionSpread(ReligionTypes eType, PlayerTypes ePlayer, CvCity* pSpreadCity)
{
	if (pSpreadCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"religion\":%d}", ePlayer, pSpreadCity->getID(), eType);
		bridgePayload("religion_spread", szArgs);
	}
	m_kPythonEventMgr.reportReligionSpread(eType, ePlayer, pSpreadCity);
}

void CvEventReporter::religionRemove(ReligionTypes eType, PlayerTypes ePlayer, CvCity* pSpreadCity)
{
	if (pSpreadCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"religion\":%d}", ePlayer, pSpreadCity->getID(), eType);
		bridgePayload("religion_remove", szArgs);
	}
	m_kPythonEventMgr.reportReligionRemove(eType, ePlayer, pSpreadCity);
}

void CvEventReporter::corporationFounded(CorporationTypes eType, PlayerTypes ePlayer)
{
	CvString szArgs;
	szArgs.Format("{\"player\":%d,\"corporation\":%d}", ePlayer, eType);
	bridgePayload("corporation_founded", szArgs);
	m_kPythonEventMgr.reportCorporationFounded(eType, ePlayer);
}

void CvEventReporter::corporationSpread(CorporationTypes eType, PlayerTypes ePlayer, CvCity* pSpreadCity)
{
	if (pSpreadCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"corporation\":%d}", ePlayer, pSpreadCity->getID(), eType);
		bridgePayload("corporation_spread", szArgs);
	}
	m_kPythonEventMgr.reportCorporationSpread(eType, ePlayer, pSpreadCity);
}

void CvEventReporter::corporationRemove(CorporationTypes eType, PlayerTypes ePlayer, CvCity* pSpreadCity)
{
	if (pSpreadCity != NULL)
	{
		CvString szArgs;
		szArgs.Format("{\"player\":%d,\"city\":%d,\"corporation\":%d}", ePlayer, pSpreadCity->getID(), eType);
		bridgePayload("corporation_remove", szArgs);
	}
	m_kPythonEventMgr.reportCorporationRemove(eType, ePlayer, pSpreadCity);
}

void CvEventReporter::goldenAge(PlayerTypes ePlayer)
{
	CvString szArgs;
	szArgs.Format("{\"player\":%d}", ePlayer);
	bridgePayload("golden_age", szArgs);
	m_kPythonEventMgr.reportGoldenAge(ePlayer);
	m_kStatistics.goldenAge(ePlayer);
}

void CvEventReporter::endGoldenAge(PlayerTypes ePlayer)
{
	CvString szArgs;
	szArgs.Format("{\"player\":%d}", ePlayer);
	bridgePayload("end_golden_age", szArgs);
	m_kPythonEventMgr.reportEndGoldenAge(ePlayer);
}

void CvEventReporter::changeWar(bool bWar, TeamTypes eTeam, TeamTypes eOtherTeam)
{
	CvString szArgs;
	szArgs.Format("{\"war\":%d,\"team\":%d,\"other_team\":%d}", bWar ? 1 : 0, eTeam, eOtherTeam);
	bridgePayload("change_war", szArgs);
	m_kPythonEventMgr.reportChangeWar(bWar, eTeam, eOtherTeam);
}

void CvEventReporter::setPlayerAlive( PlayerTypes ePlayerID, bool bNewValue )
{
	CvString szArgs;
	szArgs.Format("{\"player\":%d,\"alive\":%d}", ePlayerID, bNewValue ? 1 : 0);
	bridgePayload("set_player_alive", szArgs);
	m_kPythonEventMgr.reportSetPlayerAlive( ePlayerID, bNewValue );
}

void CvEventReporter::playerChangeStateReligion(PlayerTypes ePlayerID, ReligionTypes eNewReligion, ReligionTypes eOldReligion)
{
	CvString szArgs;
	szArgs.Format("{\"player\":%d,\"new_religion\":%d,\"old_religion\":%d}", ePlayerID, eNewReligion, eOldReligion);
	bridgePayload("player_change_state_religion", szArgs);
	m_kPythonEventMgr.reportPlayerChangeStateReligion(ePlayerID, eNewReligion, eOldReligion);
}

void CvEventReporter::playerGoldTrade(PlayerTypes eFromPlayer, PlayerTypes eToPlayer, int iAmount)
{
	CvString szArgs;
	szArgs.Format("{\"from_player\":%d,\"to_player\":%d,\"amount\":%d}", eFromPlayer, eToPlayer, iAmount);
	bridgePayload("player_gold_trade", szArgs);
	m_kPythonEventMgr.reportPlayerGoldTrade(eFromPlayer, eToPlayer, iAmount);
}

void CvEventReporter::chat(CvWString szString)
{
	CvString szText(szString);
	CvString szArgs = "{\"text\":\"";
	szArgs += jsonEscape(szText);
	szArgs += "\"}";
	bridgePayload("chat", szArgs);
	m_kPythonEventMgr.reportChat(szString);
}

void CvEventReporter::victory(TeamTypes eWinner, VictoryTypes eVictory)
{
	CvString szArgs;
	szArgs.Format("{\"team\":%d,\"victory\":%d}", eWinner, eVictory);
	bridgePayload("victory", szArgs);
	m_kPythonEventMgr.reportVictory(eWinner, eVictory);
	m_kStatistics.setVictory(eWinner, eVictory);

	// Set all human player's final total time played
	for (int i = 0; i < MAX_PLAYERS; ++i)
	{
		if (GET_PLAYER((PlayerTypes)i).isEverAlive())
		{
			m_kStatistics.setTimePlayed((PlayerTypes)i, GET_PLAYER((PlayerTypes)i).getTotalTimePlayed());
		}
	}

	// automatically report MP stats on victory
	gDLL->reportStatistics();
}

void CvEventReporter::vassalState(TeamTypes eMaster, TeamTypes eVassal, bool bVassal)
{
	CvString szArgs;
	szArgs.Format("{\"master\":%d,\"vassal\":%d,\"is_vassal\":%d}", eMaster, eVassal, bVassal ? 1 : 0);
	bridgePayload("vassal_state", szArgs);
	m_kPythonEventMgr.reportVassalState(eMaster, eVassal, bVassal);
}

void CvEventReporter::preSave()
{
	CvGameBridge::sendEvent("pre_save");
	bool bConsumed = false;
	CvGameBridge::requestCallbackConsumeTimeout("pre_save", NULL, PRESAVE_CALLBACK_TIMEOUT_MS, bConsumed);
	m_kPythonEventMgr.preSave();
}

void CvEventReporter::windowActivation(bool bActive)
{
	CvString szArgs;
	szArgs.Format("{\"active\":%d}", bActive ? 1 : 0);
	bridgePayload("window_activation", szArgs);
	m_kPythonEventMgr.reportWindowActivation(bActive);
}

void CvEventReporter::getGameStatistics(std::vector<CvStatBase*>& aStats)
{
	aStats.clear();
	aStats.push_back(new CvStatString("mapname", m_kStatistics.getMapName()));
	aStats.push_back(new CvStatInt("era", m_kStatistics.getEra()));

	// Report game params governing some server-side loops
	aStats.push_back(new CvStatInt("numplayers", MAX_CIV_PLAYERS));
	aStats.push_back(new CvStatInt("numunittypes", GC.getNumUnitInfos()));
	aStats.push_back(new CvStatInt("numbuildingtypes", GC.getNumBuildingInfos()));
	aStats.push_back(new CvStatInt("numreligiontypes", GC.getNumReligionInfos()));
}

void CvEventReporter::getPlayerStatistics(PlayerTypes ePlayer, std::vector<CvStatBase*>& aStats)
{
	aStats.clear();
	CvPlayerRecord* pRecord = m_kStatistics.getPlayerRecord(ePlayer);
	if (pRecord != NULL)
	{
		aStats.push_back(new CvStatInt("victorytype", pRecord->getVictory()));
		aStats.push_back(new CvStatInt("timeplayed", pRecord->getMinutesPlayed()));
		aStats.push_back(new CvStatInt("leader", pRecord->getLeader()-1));  // -1 because index 0 is barb
		aStats.push_back(new CvStatInt("citiesbuilt", pRecord->getNumCitiesBuilt()));
		aStats.push_back(new CvStatInt("citiesrazed", pRecord->getNumCitiesRazed()));
		aStats.push_back(new CvStatInt("goldenages", pRecord->getNumGoldenAges()));


		// Units by type
		CvString strKey;
		for (int j = 0; j < GC.getNumUnitInfos(); ++j)
		{
			strKey.format("unit_%d_built", j);
			aStats.push_back(new CvStatInt(strKey, pRecord->getNumUnitsBuilt(j)));

			strKey.format("unit_%d_killed", j);
			aStats.push_back(new CvStatInt(strKey, pRecord->getNumUnitsKilled(j)));

			strKey.format("unit_%d_lost", j);
			aStats.push_back(new CvStatInt(strKey, pRecord->getNumUnitsWasKilled(j)));
		}

		// Buildings by type
		for (int j = 0; j < GC.getNumBuildingInfos(); ++j)
		{
			strKey.format("building_%d_built", j);
			aStats.push_back(new CvStatInt(strKey, pRecord->getNumBuildingsBuilt((BuildingTypes)j)));
		}

		// Religions by type
		for (int j = 0; j < GC.getNumReligionInfos(); ++j)
		{
			strKey.format("religion_%d_founded", j);
			aStats.push_back(new CvStatInt(strKey, pRecord->getReligionFounded((ReligionTypes)j)));
		}
	}
}

void CvEventReporter::readStatistics(FDataStreamBase* pStream)
{
	m_kStatistics.reset();
	m_kStatistics.read(pStream);
}
void CvEventReporter::writeStatistics(FDataStreamBase* pStream)
{
	m_kStatistics.write(pStream);
}

