#pragma once
#include <string>
#include <vector>
#include <curl/curl.h>

struct Mod {
    std::string name;
    std::string url;
    bool selected;
};

class Downloader {
public:
    static bool download(const std::string& url, const std::string& output);
    static void downloadMods(const std::string& modsPath, const std::vector<Mod>& mods);
    static bool fileExists(const std::string& path);
    
private:
    static int xferInfoCallback(void* clientp, curl_off_t dltotal, curl_off_t dlnow, curl_off_t ultotal, curl_off_t ulnow);
};
