#pragma once
#include <string>
#include <vector>

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
    static size_t writeCallback(void* contents, size_t size, size_t nmemb, void* userp);
    static int progressCallback(void* clientp, double dltotal, double dlnow, double ultotal, double ulnow);
};
