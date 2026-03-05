/**
 * @file downloader.h
 * @brief Модуль загрузки файлов через libcurl
 */

#pragma once
#include <string>
#include <vector>
#include <curl/curl.h>
#include <atomic>
#include <mutex>

/**
 * @struct Mod
 * @brief Информация о моде для загрузки
 */
struct Mod {
    std::string name;      ///< Название мода
    std::string url;       ///< URL для скачивания
    bool selected;         ///< Выбран ли мод для установки
};

/**
 * @struct DownloadTask
 * @brief Задача на загрузку файла
 */
struct DownloadTask {
    std::string url;       ///< URL источника
    std::string output;    ///< Путь для сохранения
};

/**
 * @struct DownloadStats
 * @brief Статистика многопоточной загрузки
 */
struct DownloadStats {
    std::atomic<int> completed{0};           ///< Завершено задач
    std::atomic<int> failed{0};              ///< Провалено задач
    std::atomic<int> skipped{0};             ///< Пропущено (уже существуют)
    std::atomic<long long> bytesDownloaded{0}; ///< Скачано байт
    std::mutex mutex;                        ///< Мьютекс для синхронизации
};

/**
 * @class Downloader
 * @brief Класс для загрузки файлов с поддержкой многопоточности
 */
class Downloader {
public:
    /**
     * @brief Загружает файл по URL
     * @param url URL источника
     * @param output Путь для сохранения
     * @return true если успешно
     */
    static bool download(const std::string& url, const std::string& output);
    
    /**
     * @brief Загружает список модов
     * @param modsPath Путь к папке модов
     * @param mods Список модов для загрузки
     */
    static void downloadMods(const std::string& modsPath, const std::vector<Mod>& mods);
    
    /**
     * @brief Проверяет существование файла
     * @param path Путь к файлу
     * @return true если файл существует
     */
    static bool fileExists(const std::string& path);
    
    /**
     * @brief Загружает файлы параллельно в несколько потоков
     * @param tasks Список задач на загрузку
     * @param numThreads Количество потоков
     * @param stats Статистика загрузки
     */
    static void downloadParallel(const std::vector<DownloadTask>& tasks, int numThreads, DownloadStats& stats);
    
private:
    /**
     * @brief Callback для отслеживания прогресса загрузки
     */
    static int xferInfoCallback(void* clientp, curl_off_t dltotal, curl_off_t dlnow, curl_off_t ultotal, curl_off_t ulnow);
};
