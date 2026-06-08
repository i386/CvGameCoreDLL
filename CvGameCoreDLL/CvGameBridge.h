#pragma once

#ifndef CvGameBridge_h
#define CvGameBridge_h

namespace CvGameBridge
{
	void init();
	void shutdown();
	void poll();
	bool isEnabled();
	void sendEvent(const char* szName, const char* szArgsJson = NULL);
	void sendCallbackMirror(const char* szName, const char* szArgsJson = NULL);
	bool requestCallbackConsume(const char* szName, const char* szArgsJson, bool& bConsumed);
	bool requestCallbackConsumeTimeout(const char* szName, const char* szArgsJson, unsigned int uiTimeoutMs, bool& bConsumed);
	bool requestCallbackBool(const char* szName, const char* szArgsJson, bool& bValue);
	bool requestCallbackInt(const char* szName, const char* szArgsJson, int& iValue);
}

#endif
