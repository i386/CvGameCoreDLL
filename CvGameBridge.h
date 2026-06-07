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
}

#endif
