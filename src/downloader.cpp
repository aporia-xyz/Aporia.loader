/**
 * @file downloader.cpp
 * @brief Реализация модуля загрузки файлов
 */

#include "downloader.h"
#include "utils.h"
#include <curl/curl.h>
#include <fstream>
#include <iostream>
#include <filesystem>
#include <thread>
#include <queue>
#include <mutex>
#include <condition_variable>

namespace fs = std::filesystem;

int Downloader::xferInfoCallback(void* clientp, curl_off_t dltotal, curl_off_t dlnow, curl_off_t ultotal, curl_off_t ulnow) {
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
        
        std::string rawFilename = mod.url.substr(mod.url.find_last_of('/') + 1);
        
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

void Downloader::downloadParallel(const std::vector<DownloadTask>& tasks, int numThreads, DownloadStats& stats) {
    std::queue<DownloadTask> taskQueue;
    std::mutex queueMutex;
    std::condition_variable cv;
    bool done = false;
    
    for (const auto& task : tasks) {
        taskQueue.push(task);
    }
    
    auto worker = [&]() {
        CURL* curl = curl_easy_init();
        if (!curl) return;
        
        while (true) {
            DownloadTask task;
            
            {
                std::unique_lock<std::mutex> lock(queueMutex);
                if (taskQueue.empty()) {
                    break;
                }
                task = taskQueue.front();
                taskQueue.pop();
            }
            
            if (fs::exists(task.output)) {
                stats.skipped++;
                stats.completed++;
                continue;
            }
            
            FILE* fp = fopen(task.output.c_str(), "wb");
            if (!fp) {
                stats.failed++;
                stats.completed++;
                continue;
            }
            
            curl_easy_setopt(curl, CURLOPT_URL, task.url.c_str());
            curl_easy_setopt(curl, CURLOPT_WRITEDATA, fp);
            curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
            curl_easy_setopt(curl, CURLOPT_NOPROGRESS, 1L);
            curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0L);
            curl_easy_setopt(curl, CURLOPT_TIMEOUT, 30L);
            
            CURLcode res = curl_easy_perform(curl);
            
            long fileSize = 0;
            fseek(fp, 0, SEEK_END);
            fileSize = ftell(fp);
            fclose(fp);
            
            if (res == CURLE_OK && fileSize > 0) {
                stats.bytesDownloaded += fileSize;
            } else {
                fs::remove(task.output);
                stats.failed++;
            }
            
            stats.completed++;
        }
        
        curl_easy_cleanup(curl);
    };
    
    std::vector<std::thread> threads;
    for (int i = 0; i < numThreads; i++) {
        threads.emplace_back(worker);
    }
    
    for (auto& t : threads) {
        t.join();
    }
}
