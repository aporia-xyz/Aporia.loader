#include "downloader.h"
#include "utils.h"
#include <curl/curl.h>
#include <fstream>
#include <iostream>
#include <filesystem>

namespace fs = std::filesystem;

int Downloader::xferInfoCallback(void* clientp, curl_off_t dltotal, curl_off_t dlnow, curl_off_t ultotal, curl_off_t ulnow) {
    if (dltotal > 0) {
        int percent = (int)((dlnow * 100) / dltotal);
        std::cout << "\r" << Utils::Color::CYAN << "Загрузка: [";
        int bars = percent / 2;
        for (int i = 0; i < 50; i++) {
            if (i < bars) std::cout << "█";
            else std::cout << "░";
        }
        std::cout << "] " << percent << "%" << Utils::Color::RESET << std::flush;
    }
    return 0;
}

bool Downloader::download(const std::string& url, const std::string& output) {
    CURL* curl = curl_easy_init();
    if (!curl) return false;
    
    FILE* fp = fopen(output.c_str(), "wb");
    if (!fp) {
        curl_easy_cleanup(curl);
        return false;
    }
    
    curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, fp);
    curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
    curl_easy_setopt(curl, CURLOPT_NOPROGRESS, 0L);
    curl_easy_setopt(curl, CURLOPT_XFERINFOFUNCTION, xferInfoCallback);
    curl_easy_setopt(curl, CURLOPT_XFERINFODATA, nullptr);
    curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0L);
    
    CURLcode res = curl_easy_perform(curl);
    
    fclose(fp);
    curl_easy_cleanup(curl);
    
    std::cout << "\n";
    return res == CURLE_OK;
}

bool Downloader::fileExists(const std::string& path) {
    return fs::exists(path);
}

void Downloader::downloadMods(const std::string& modsPath, const std::vector<Mod>& mods) {
    fs::create_directories(modsPath);
    
    CURL* curl = curl_easy_init();
    if (!curl) return;
    
    for (const auto& mod : mods) {
        if (!mod.selected) continue;
        
        // Берем имя файла из URL
        std::string rawFilename = mod.url.substr(mod.url.find_last_of('/') + 1);
        
        // ДЕКОДИРУЕМ %2B в + и прочее
        int outLength;
        char* decoded = curl_easy_unescape(curl, rawFilename.c_str(), 0, &outLength);
        std::string filename(decoded, outLength);
        curl_free(decoded);
        
        std::string output = modsPath + "/" + filename;
        
        if (fileExists(output)) {
            std::cout << Utils::Color::GREEN << "✓ " << mod.name << " уже загружен\n" << Utils::Color::RESET;
            continue;
        }
        
        std::cout << Utils::Color::YELLOW << "⬇ Загрузка " << mod.name << "...\n" << Utils::Color::RESET;
        if (download(mod.url, output)) {
            std::cout << Utils::Color::GREEN << "✓ " << mod.name << " загружен!\n" << Utils::Color::RESET;
        } else {
            std::cout << Utils::Color::RED << "✗ Ошибка загрузки " << mod.name << "\n" << Utils::Color::RESET;
        }
    }
    
    curl_easy_cleanup(curl);
}
