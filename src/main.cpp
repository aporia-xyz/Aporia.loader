#include "utils.h"
#include "config.h"
#include "downloader.h"
#include "resource_extractor.h"
#include <iostream>
#include <filesystem>
#include <vector>
#include <sstream>
#include <fstream>

namespace fs = std::filesystem;

// Uncomment after running embed_resources.py
// #include "embedded_resources.h"

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

void extractEmbeddedResources(const Config& config) {
    /* Uncomment after running embed_resources.py
    std::cout << Utils::Color::YELLOW << "\n📦 Извлечение встроенных ресурсов...\n" << Utils::Color::RESET;
    
    auto resources = EmbeddedResources::getAllResources();
    for (const auto& res : resources) {
        std::string fullPath = config.installPath + "/" + res.path;
        
        if (fs::exists(fullPath)) {
            std::cout << Utils::Color::GREEN << "✓ " << res.path << " уже существует\n" << Utils::Color::RESET;
            continue;
        }
        
        if (ResourceExtractor::extractResource(res.data_b64, fullPath)) {
            std::cout << Utils::Color::GREEN << "✓ Извлечено: " << res.path << "\n" << Utils::Color::RESET;
        } else {
            std::cout << Utils::Color::RED << "✗ Ошибка: " << res.path << "\n" << Utils::Color::RESET;
        }
    }
    */
    
    // Temporary: copy from src/assets
    std::string assetsPath = "src/assets/assets";
    std::string librariesPath = "src/assets/libraries";
    std::string jarPath = "src/assets/Fabric 1.21.11.jar";
    std::string jsonPath = "src/assets/Fabric 1.21.11.json";
    
    copyDirectory(assetsPath, config.installPath + "/assets");
    copyDirectory(librariesPath, config.installPath + "/libraries");
    
    std::string versionsPath = config.installPath + "/versions/Fabric 1.21.11";
    fs::create_directories(versionsPath);
    
    std::string jarDst = versionsPath + "/Fabric 1.21.11.jar";
    std::string jsonDst = versionsPath + "/Fabric 1.21.11.json";
    
    if (fs::exists(jarPath) && !fs::exists(jarDst)) {
        fs::copy_file(jarPath, jarDst);
        std::cout << Utils::Color::GREEN << "✓ Скопирован Fabric jar\n" << Utils::Color::RESET;
    } else if (fs::exists(jarDst)) {
        std::cout << Utils::Color::GREEN << "✓ Fabric jar уже существует\n" << Utils::Color::RESET;
    }
    
    if (fs::exists(jsonPath) && !fs::exists(jsonDst)) {
        fs::copy_file(jsonPath, jsonDst);
        std::cout << Utils::Color::GREEN << "✓ Скопирован Fabric json\n" << Utils::Color::RESET;
    } else if (fs::exists(jsonDst)) {
        std::cout << Utils::Color::GREEN << "✓ Fabric json уже существует\n" << Utils::Color::RESET;
    }
}

void setupGameFiles(const Config& config) {
    extractEmbeddedResources(config);
}

std::vector<Mod> getMods() {
    return {
        // {"Iris Shaders", "https://cdn.modrinth.com/data/YL57xq9U/versions/TSXvi2yD/iris-fabric-1.10.6%2Bmc1.21.11.jar", true},
        // {"Sodium", "https://cdn.modrinth.com/data/AANobbMI/versions/ZPWbiWXz/sodium-fabric-0.8.6%2Bmc1.21.11.jar", true},
        {"Mod Menu", "https://cdn.modrinth.com/data/mOgUt4GM/versions/JWQVh32x/modmenu-17.0.0-beta.2.jar", true},
        // {"Sodium Extra", "https://cdn.modrinth.com/data/PtjYWJkn/versions/yqY1efrC/sodium-extra-fabric-0.8.3%2Bmc1.21.11.jar", true},
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

void launchMinecraft(const Config& config) {
    std::string javaCmd = findJava();
    std::string libsPath = config.installPath + "\\libraries";
    
    // 1. Собираем все либы в одну строку
    std::string cp_string = config.installPath + "\\versions\\Fabric 1.21.11\\Fabric 1.21.11.jar";
    
    for (const auto& entry : fs::recursive_directory_iterator(libsPath)) {
        if (entry.is_regular_file() && entry.path().extension() == ".jar") {
            cp_string += ";" + entry.path().string();
        }
    }

    // 2. Запихиваем этот гигантский текст в переменную окружения процесса
    // Это не портит систему, переменная живет только пока работает твой лаунчер
    _putenv_s("CLASSPATH", cp_string.c_str());

    // 3. Теперь команда запуска ОЧЕНЬ короткая, т.к. Java сама возьмет либы из CLASSPATH
    std::stringstream cmd;
    // Оборачиваем в cmd /k, чтобы видеть лог, если моды битые!
    cmd << "cmd /k \"\"" << javaCmd << "\""; 
    cmd << " -Xmx" << config.ramMB << "M";
    if (config.devMode) cmd << " -noverify";
    cmd << " net.fabricmc.loader.impl.launch.knot.KnotClient";
    cmd << " --gameDir \"" << config.installPath << "\"";
    cmd << " --version \"Fabric 1.21.11\"";
    cmd << " --assetsDir \"" << config.installPath << "\\assets\"";
    cmd << " --assetIndex 29";
    cmd << " --username " << config.username << "\"";
    
    std::cout << Utils::Color::CYAN << "🚀 Запуск через CLASSPATH env..." << Utils::Color::RESET << std::endl;
    
    // 4. Огонь!
    system(cmd.str().c_str());
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
            
            std::string modsPath = config.installPath + "/mods";
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
