/**
 * @file main.cpp
 * @brief Aporia Minecraft Loader - главный файл лаунчера
 */

#include "utils.h"
#include "config.h"
#include "downloader.h"
#include "json.hpp"
#include <iostream>
#include <filesystem>
#include <vector>
#include <sstream>
#include <fstream>
#include <thread>
#include <chrono>
#include <iomanip>

using json = nlohmann::json;
namespace fs = std::filesystem;

/**
 * @brief Копирует директорию рекурсивно
 * @param src Исходная директория
 * @param dst Целевая директория
 */
void copyDirectory(const fs::path& src, const fs::path& dst) {
    if (!fs::exists(src)) return;
    if (fs::exists(dst)) {
        std::cout << Utils::Color::GREEN << "✓ " << dst.filename().string() << " уже существует\n" << Utils::Color::RESET;
        return;
    }
    
    fs::create_directories(dst);
    for (const auto& entry : fs::recursive_directory_iterator(src)) {
        const auto& path = entry.path();
        auto relPath = fs::relative(path, src);
        auto dstPath = dst / relPath;
        if (fs::is_directory(path)) {
            fs::create_directories(dstPath);
        } else {
            fs::copy_file(path, dstPath, fs::copy_options::skip_existing);
        }
    }
}

/**
 * @brief Ищет установленную Java в системе
 * @return Путь к java.exe или "java" если не найдена
 */
std::string findJava() {
    std::vector<std::string> javaPaths = {
        "C:\\Program Files\\Java\\jdk-26\\bin\\java.exe",
        "C:\\Program Files\\Java\\jdk-25.0.2\\bin\\java.exe",
        "C:\\Program Files\\Java\\jdk-21.0.10\\bin\\java.exe",
        "C:\\Program Files\\Java\\latest\\bin\\java.exe"
    };
    
    for (const auto& path : javaPaths) {
        if (fs::exists(path)) {
            return path;
        }
    }
    
    return "java";
}

/**
 * @brief Определяет имя ОС для Minecraft
 * @return "windows", "osx" или "linux"
 */
std::string getOsName() {
#ifdef _WIN32
    return "windows";
#elif __APPLE__
    return "osx";
#else
    return "linux";
#endif
}

/**
 * @brief Конвертирует Maven координаты в URL
 * @param maven Maven координаты (group:artifact:version)
 * @param classifier Классификатор (например "natives-windows")
 * @param baseUrl Базовый URL репозитория
 * @return Полный URL для скачивания
 */
std::string mavenToUrl(const std::string& maven, const std::string& classifier, const std::string& baseUrl) {
    std::string cleanMaven = maven;
    size_t lastColon = maven.rfind(':');
    size_t secondColon = maven.find(':', maven.find(':') + 1);
    
    if (lastColon != secondColon && lastColon != std::string::npos) {
        cleanMaven = maven.substr(0, lastColon);
    }
    
    size_t colon1 = cleanMaven.find(':');
    size_t colon2 = cleanMaven.find(':', colon1 + 1);
    
    if (colon1 == std::string::npos || colon2 == std::string::npos) return "";
    
    std::string group = cleanMaven.substr(0, colon1);
    std::string artifact = cleanMaven.substr(colon1 + 1, colon2 - colon1 - 1);
    std::string version = cleanMaven.substr(colon2 + 1);
    
    for (char& c : group) {
        if (c == '.') c = '/';
    }
    
    std::string filename = artifact + "-" + version;
    if (!classifier.empty()) {
        filename += "-" + classifier;
    }
    filename += ".jar";
    
    return baseUrl + group + "/" + artifact + "/" + version + "/" + filename;
}

std::string mavenToPath(const std::string& maven, const std::string& classifier, const std::string& basePath) {
    std::string cleanMaven = maven;
    size_t lastColon = maven.rfind(':');
    size_t secondColon = maven.find(':', maven.find(':') + 1);
    
    if (lastColon != secondColon && lastColon != std::string::npos) {
        cleanMaven = maven.substr(0, lastColon);
    }
    
    size_t colon1 = cleanMaven.find(':');
    size_t colon2 = cleanMaven.find(':', colon1 + 1);
    
    std::string group = cleanMaven.substr(0, colon1);
    std::string artifact = cleanMaven.substr(colon1 + 1, colon2 - colon1 - 1);
    std::string version = cleanMaven.substr(colon2 + 1);
    
    for (char& c : group) {
        if (c == '.') c = '/';
    }
    
    std::string filename = artifact + "-" + version;
    if (!classifier.empty()) {
        filename += "-" + classifier;
    }
    filename += ".jar";
    
    return basePath + "/" + group + "/" + artifact + "/" + version + "/" + filename;
}

void downloadAllResources(const Config& config) {
    std::string jsonPath = config.installPath + "/versions/Fabric 1.21.11/Fabric 1.21.11.json";
    
    if (!fs::exists(jsonPath)) {
        std::cout << Utils::Color::RED << "✗ JSON не найден\n" << Utils::Color::RESET;
        return;
    }
    
    std::ifstream file(jsonPath);
    json versionJson;
    file >> versionJson;
    
    std::string libsPath = config.installPath + "/libraries";
    std::string osName = getOsName();
    
    std::vector<DownloadTask> allTasks;
    
    if (versionJson.contains("libraries")) {
        for (const auto& lib : versionJson["libraries"]) {
            if (!lib.contains("name")) continue;
            
            std::string name = lib["name"];
            
            if (name.find("ru.legacylauncher") != std::string::npos) continue;
            
            if (lib.contains("rules")) {
                bool allowed = false;
                for (const auto& rule : lib["rules"]) {
                    if (rule["action"] == "allow") {
                        if (rule.contains("os")) {
                            if (rule["os"]["name"] == osName) allowed = true;
                        } else {
                            allowed = true;
                        }
                    }
                }
                if (!allowed) continue;
            }
            
            std::string baseUrl = "https://libraries.minecraft.net/";
            if (lib.contains("url")) baseUrl = lib["url"];
            
            std::string classifier = "";
            std::string cleanName = name;
            size_t colonPos = name.rfind(':');
            size_t secondColonPos = name.find(':', name.find(':') + 1);
            if (colonPos != secondColonPos && colonPos != std::string::npos) {
                classifier = name.substr(colonPos + 1);
                cleanName = name.substr(0, colonPos);
            }
            
            std::string url = mavenToUrl(cleanName, classifier, baseUrl);
            std::string localPath = mavenToPath(cleanName, classifier, libsPath);
            fs::create_directories(fs::path(localPath).parent_path());
            allTasks.push_back({url, localPath});
        }
    }
    
    if (versionJson.contains("assetIndex")) {
        auto assetIndex = versionJson["assetIndex"];
        std::string assetIndexUrl = assetIndex["url"];
        std::string assetIndexId = assetIndex["id"];
        std::string indexPath = config.installPath + "/assets/indexes/" + assetIndexId + ".json";
        
        fs::create_directories(fs::path(indexPath).parent_path());
        if (!fs::exists(indexPath)) {
            Downloader::download(assetIndexUrl, indexPath);
        }
        
        std::ifstream indexFile(indexPath);
        json indexJson;
        indexFile >> indexJson;
        
        if (indexJson.contains("objects")) {
            for (auto& [key, value] : indexJson["objects"].items()) {
                std::string hash = value["hash"];
                std::string subdir = hash.substr(0, 2);
                std::string objectPath = config.installPath + "/assets/objects/" + subdir + "/" + hash;
                fs::create_directories(fs::path(objectPath).parent_path());
                
                std::string url = "https://resources.download.minecraft.net/" + subdir + "/" + hash;
                allTasks.push_back({url, objectPath});
            }
        }
    }
    
    int totalFiles = allTasks.size();
    std::cout << Utils::Color::YELLOW << "\n📦 Всего файлов: " << totalFiles << "\n" << Utils::Color::RESET;
    
    DownloadStats stats;
    auto startTime = std::chrono::steady_clock::now();
    
    std::thread downloadThread([&]() {
        Downloader::downloadParallel(allTasks, 8, stats);
    });
    
    while (stats.completed < totalFiles) {
        auto now = std::chrono::steady_clock::now();
        auto elapsed = std::chrono::duration_cast<std::chrono::seconds>(now - startTime).count();
        
        int current = stats.completed.load();
        int percent = (current * 100) / totalFiles;
        
        double speed = elapsed > 0 ? (stats.bytesDownloaded.load() / 1024.0 / 1024.0) / elapsed : 0;
        
        int remaining = 0;
        if (current > 0 && elapsed > 0) {
            double avgTimePerFile = (double)elapsed / current;
            remaining = (int)(avgTimePerFile * (totalFiles - current));
        }
        
        std::cout << "\r" << Utils::Color::CYAN << "[" << current << "/" << totalFiles << "] ";
        int bars = (current * 50) / totalFiles;
        for (int i = 0; i < 50; i++) {
            std::cout << (i < bars ? "█" : "░");
        }
        std::cout << " " << percent << "% | " 
                  << std::fixed << std::setprecision(2) << speed << " MB/s | "
                  << remaining << "s" << Utils::Color::RESET << std::flush;
        
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }
    
    downloadThread.join();
    
    std::cout << "\n" << Utils::Color::GREEN << "✓ Загружено: " << (totalFiles - stats.skipped.load()) 
              << ", пропущено: " << stats.skipped.load() << "\n" << Utils::Color::RESET;
}

void downloadAssetsFromJson(const Config& config) {
    std::string jsonPath = config.installPath + "/versions/Fabric 1.21.11/Fabric 1.21.11.json";
    
    if (!fs::exists(jsonPath)) {
        std::cout << Utils::Color::RED << "✗ JSON не найден\n" << Utils::Color::RESET;
        return;
    }
    
    // Читаем version JSON
    std::ifstream file(jsonPath);
    json versionJson;
    file >> versionJson;
    
    if (!versionJson.contains("assetIndex")) {
        std::cout << Utils::Color::RED << "✗ assetIndex не найден\n" << Utils::Color::RESET;
        return;
    }
    
    auto assetIndex = versionJson["assetIndex"];
    std::string assetIndexUrl = assetIndex["url"];
    std::string assetIndexId = assetIndex["id"];
    
    std::cout << Utils::Color::YELLOW << "\n🎨 Asset index: " << assetIndexId << "\n" << Utils::Color::RESET;
    
    // Скачиваем asset index
    std::string indexPath = config.installPath + "/assets/indexes/" + assetIndexId + ".json";
    fs::create_directories(fs::path(indexPath).parent_path());
    
    if (!fs::exists(indexPath)) {
        std::cout << Utils::Color::CYAN << "⬇ Загрузка asset index...\n" << Utils::Color::RESET;
        if (!Downloader::download(assetIndexUrl, indexPath)) {
            std::cout << Utils::Color::RED << "✗ Не удалось загрузить asset index\n" << Utils::Color::RESET;
            return;
        }
    }
    
    // Парсим asset index
    std::ifstream indexFile(indexPath);
    json indexJson;
    indexFile >> indexJson;
    
    if (!indexJson.contains("objects")) {
        std::cout << Utils::Color::RED << "✗ objects не найдены в asset index\n" << Utils::Color::RESET;
        return;
    }
    
    auto objects = indexJson["objects"];
    int totalAssets = objects.size();
    
    std::cout << Utils::Color::YELLOW << "📦 Ассетов для загрузки: " << totalAssets << "\n" << Utils::Color::RESET;
    
    int current = 0;
    int downloaded = 0;
    int skipped = 0;
    
    for (auto& [key, value] : objects.items()) {
        std::string hash = value["hash"];
        std::string subdir = hash.substr(0, 2);
        std::string objectPath = config.installPath + "/assets/objects/" + subdir + "/" + hash;
        
        current++;
        int percent = (current * 100) / totalAssets;
        
        if (fs::exists(objectPath)) {
            skipped++;
        } else {
            fs::create_directories(fs::path(objectPath).parent_path());
            
            std::string url = "https://resources.download.minecraft.net/" + subdir + "/" + hash;
            if (Downloader::download(url, objectPath)) {
                downloaded++;
            }
        }
        
        // Обновляем прогресс каждые 10 файлов
        if (current % 10 == 0 || current == totalAssets) {
            std::cout << "\r" << Utils::Color::CYAN << "[" << current << "/" << totalAssets << "] ";
            int bars = (current * 50) / totalAssets;
            for (int i = 0; i < 50; i++) {
                if (i < bars) std::cout << "█";
                else std::cout << "░";
            }
            std::cout << " " << percent << "%" << Utils::Color::RESET << std::flush;
        }
    }
    
    std::cout << "\n" << Utils::Color::GREEN << "✓ Ассеты: загружено " << downloaded << ", пропущено " << skipped << "\n" << Utils::Color::RESET;
}

void setupGameFiles(const Config& config) {
    std::cout << Utils::Color::YELLOW << "\n📦 Загрузка файлов с GitHub...\n" << Utils::Color::RESET;
    
    std::string versionsPath = config.installPath + "/versions/Fabric 1.21.11";
    std::string jarPath = versionsPath + "/Fabric 1.21.11.jar";
    std::string jsonPath = versionsPath + "/Fabric 1.21.11.json";
    
    fs::create_directories(versionsPath);
    
    // Скачиваем jar
    if (!fs::exists(jarPath)) {
        std::cout << Utils::Color::YELLOW << "⬇ Загрузка Fabric jar...\n" << Utils::Color::RESET;
        if (Downloader::download("https://raw.githubusercontent.com/aporia-xyz/Aporia.loader/refs/heads/main/versions/Fabric%201.21.11/Fabric%201.21.11.jar", jarPath)) {
            std::cout << Utils::Color::GREEN << "✓ Fabric jar загружен\n" << Utils::Color::RESET;
        }
    } else {
        std::cout << Utils::Color::GREEN << "✓ Fabric jar уже существует\n" << Utils::Color::RESET;
    }
    
    // Скачиваем json
    if (!fs::exists(jsonPath)) {
        std::cout << Utils::Color::YELLOW << "⬇ Загрузка Fabric json...\n" << Utils::Color::RESET;
        if (Downloader::download("https://raw.githubusercontent.com/aporia-xyz/Aporia.loader/refs/heads/main/versions/Fabric%201.21.11/Fabric%201.21.11.json", jsonPath)) {
            std::cout << Utils::Color::GREEN << "✓ Fabric json загружен\n" << Utils::Color::RESET;
        }
    } else {
        std::cout << Utils::Color::GREEN << "✓ Fabric json уже существует\n" << Utils::Color::RESET;
    }
    
    // Качаем libraries и assets с общим прогрессом
    downloadAllResources(config);
}

std::vector<Mod> getMods() {
    return {
        {"Mod Menu", "https://cdn.modrinth.com/data/mOgUt4GM/versions/JWQVh32x/modmenu-17.0.0-beta.2.jar", true},
        {"3D Skin Layers", "https://cdn.modrinth.com/data/zV5r3pPn/versions/JS9deRtw/skinlayers3d-fabric-1.10.2-mc1.21.11.jar", true},
        {"Sound Physics Remastered", "https://cdn.modrinth.com/data/qyVF9oeo/versions/pfqxi9qs/sound-physics-remastered-fabric-1.21.11-1.5.1.jar", true},
        {"Cloth Config", "https://cdn.modrinth.com/data/9s6osm5g/versions/xuX40TN5/cloth-config-21.11.153-fabric.jar", true}
    };
}

void selectMods(std::vector<Mod>& mods) {
    Utils::clearScreen();
    Utils::printHeader();
    
    std::cout << Utils::Color::MAGENTA << "\n📦 Выбор модов\n" << Utils::Color::RESET;
    std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n";
    
    for (size_t i = 0; i < mods.size(); i++) {
        std::cout << (i + 1) << ". [" << (mods[i].selected ? Utils::Color::GREEN + "✓" : Utils::Color::RED + "✗") 
                  << Utils::Color::RESET << "] " << mods[i].name << "\n";
    }
    
    std::cout << "\nВведите номер мода для переключения (0 для продолжения): ";
    int choice;
    std::cin >> choice;
    std::cin.ignore();
    
    if (choice > 0 && choice <= (int)mods.size()) {
        mods[choice - 1].selected = !mods[choice - 1].selected;
        selectMods(mods);
    }
}

/**
 * @brief Распаковывает native библиотеки для текущей ОС
 * @param config Конфигурация лаунчера
 */
void extractNatives(const Config& config) {
    std::string libsPath = config.installPath + "/libraries";
    std::string nativesDir = config.installPath + "/versions/Fabric 1.21.11/natives";
    
    fs::create_directories(nativesDir);
    
    std::cout << Utils::Color::YELLOW << "📦 Распаковка natives...\n" << Utils::Color::RESET;
    
    std::string osName = getOsName();
    std::string nativePattern;
    
    if (osName == "windows") {
        nativePattern = "natives-windows";
    } else if (osName == "osx") {
        nativePattern = "natives-macos";
    } else {
        nativePattern = "natives-linux";
    }
    
    if (fs::exists(libsPath)) {
        for (const auto& entry : fs::recursive_directory_iterator(libsPath)) {
            if (entry.is_regular_file() && entry.path().extension() == ".jar") {
                std::string filename = entry.path().filename().string();
                if (filename.find(nativePattern) != std::string::npos) {
#ifdef _WIN32
                    std::string cmd = "powershell -Command \"Add-Type -AssemblyName System.IO.Compression.FileSystem; [System.IO.Compression.ZipFile]::ExtractToDirectory('" + entry.path().string() + "', '" + nativesDir + "')\" 2>nul";
#else
                    std::string cmd = "unzip -o -q \"" + entry.path().string() + "\" -d \"" + nativesDir + "\" 2>/dev/null";
#endif
                    system(cmd.c_str());
                }
            }
        }
    }
    
    std::cout << Utils::Color::GREEN << "✓ Natives распакованы\n" << Utils::Color::RESET;
}

/**
 * @brief Запускает Minecraft с настроенными параметрами
 * @param config Конфигурация лаунчера
 */
void launchMinecraft(const Config& config) {
    std::string javaCmd = findJava();
    std::string gameDir = config.installPath + "/game";
    std::string libsPath = config.installPath + "/libraries";
    std::string assetsPath = config.installPath + "/assets";
    std::string nativesDir = config.installPath + "/versions/Fabric 1.21.11/natives";
    
    fs::create_directories(gameDir);
    
#ifdef _WIN32
    std::string cp_string = config.installPath + "\\versions\\Fabric 1.21.11\\Fabric 1.21.11.jar";
    char separator = ';';
#else
    std::string cp_string = config.installPath + "/versions/Fabric 1.21.11/Fabric 1.21.11.jar";
    char separator = ':';
#endif
    
    if (fs::exists(libsPath)) {
        for (const auto& entry : fs::recursive_directory_iterator(libsPath)) {
            if (entry.is_regular_file() && entry.path().extension() == ".jar") {
                cp_string += separator + entry.path().string();
            }
        }
    }

#ifdef _WIN32
    _putenv_s("CLASSPATH", cp_string.c_str());
#else
    setenv("CLASSPATH", cp_string.c_str(), 1);
#endif

    std::stringstream cmd;
#ifdef _WIN32
    cmd << "start \"Minecraft\" /B \"" << javaCmd << "\"";
#else
    cmd << "\"" << javaCmd << "\"";
#endif
    cmd << " -Xmx" << config.ramMB << "M";
    cmd << " -Djava.library.path=\"" << nativesDir << "\"";
    if (config.devMode) cmd << " -noverify";
    cmd << " net.fabricmc.loader.impl.launch.knot.KnotClient";
    cmd << " --gameDir \"" << gameDir << "\"";
    cmd << " --version \"Fabric 1.21.11\"";
    cmd << " --assetsDir \"" << assetsPath << "\"";
    cmd << " --assetIndex 29";
    cmd << " --username " << config.username;
    
    std::cout << Utils::Color::CYAN << "🚀 Запуск Minecraft...\n" << Utils::Color::RESET;
    std::cout << Utils::Color::WHITE << "Game Dir: " << gameDir << "\n" << Utils::Color::RESET;
    
#ifdef _WIN32
    system(cmd.str().c_str());
#else
    std::string launchCmd = cmd.str() + " &";
    system(launchCmd.c_str());
#endif
}


void drawMenu(int selected) {
    Utils::clearScreen();
    Utils::printHeader();
    
    std::vector<std::string> items = {
        "🚀 Запуск",
        "⚙️  Настройки", 
        "📦 Выбор модов",
        "❌ Выход"
    };
    
    std::cout << "\n";
    for (size_t i = 0; i < items.size(); i++) {
        if (i == selected) {
            std::cout << "        " << Utils::Color::GREEN << Utils::Color::BOLD 
                     << "▶ " << items[i] << " ◀" << Utils::Color::RESET << "\n\n";
        } else {
            std::cout << "          " << Utils::Color::WHITE << items[i] << Utils::Color::RESET << "\n\n";
        }
    }
    
    std::cout << "\n  " << Utils::Color::CYAN << "Используй ↑↓ или 1-4 для выбора" << Utils::Color::RESET << "\n";
}

int main() {
    Utils::enableColors();
    
    Config config;
    config.load();
    
    int selected = 0;
    int choice = -1;
    
    while (choice == -1) {
        drawMenu(selected);
        
        int key = Utils::getKeyPress();
        
        if (key >= '1' && key <= '4') {
            choice = key - '1';
        } else if (key == 'w' || key == 72) {
            selected = (selected - 1 + 4) % 4;
        } else if (key == 's' || key == 80) {
            selected = (selected + 1) % 4;
        } else if (key == '\r' || key == '\n') {
            choice = selected;
        }
    }
    
    switch (choice) {
        case 0: {
            setupGameFiles(config);
            extractNatives(config);
            
            std::string modsPath = config.installPath + "/game/mods";
            fs::create_directories(modsPath);
            
            std::string fabricApi = modsPath + "/fabric-api-0.141.2+1.21.11.jar";
            if (!Downloader::fileExists(fabricApi)) {
                std::cout << Utils::Color::YELLOW << "\n⬇ Загрузка Fabric API...\n" << Utils::Color::RESET;
                Downloader::download("https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/0.141.2%2B1.21.11/fabric-api-0.141.2%2B1.21.11.jar", fabricApi);
            }
            
            std::string aporia = modsPath + "/Aporia-0.4.jar";
            if (!Downloader::fileExists(aporia)) {
                std::cout << Utils::Color::YELLOW << "\n⬇ Загрузка Aporia...\n" << Utils::Color::RESET;
                Downloader::download("https://github.com/dakychan/Aporia/releases/download/0.4/Aporia-0.4.jar", aporia);
            }
            
            auto mods = getMods();
            Downloader::downloadMods(modsPath, mods);
            
            launchMinecraft(config);
            break;
        }
        case 1:
            config.setup();
            main();
            break;
        case 2: {
            auto mods = getMods();
            selectMods(mods);
            main();
            break;
        }
        case 3:
            return 0;
    }
    
    return 0;
}
