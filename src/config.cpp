/**
 * @file config.cpp
 * @brief Реализация модуля конфигурации
 */

#include "config.h"
#include "utils.h"
#include <iostream>
#include <filesystem>
#include <sstream>

namespace fs = std::filesystem;

Config::Config() {
    installPath = Utils::getDefaultPath();
    ramMB = 4096;
    username = "Player";
    devMode = false;
    
    configFile = installPath + "/config.txt";
}

std::string Config::trim(const std::string& str) {
    size_t first = str.find_first_not_of(" \t\r\n");
    if (first == std::string::npos) return "";
    size_t last = str.find_last_not_of(" \t\r\n");
    return str.substr(first, last - first + 1);
}

void Config::load() {
    std::ifstream file(configFile);
    if (!file.is_open()) return;
    
    std::string line;
    while (std::getline(file, line)) {
        size_t pos = line.find('=');
        if (pos != std::string::npos) {
            std::string key = trim(line.substr(0, pos));
            std::string value = trim(line.substr(pos + 1));
            data[key] = value;
        }
    }
    
    if (data.count("path")) installPath = data["path"];
    if (data.count("ram")) ramMB = std::stoi(data["ram"]);
    if (data.count("username")) username = data["username"];
    if (data.count("devmode")) devMode = (data["devmode"] == "true");
}

void Config::save() {
    fs::create_directories(fs::path(configFile).parent_path());
    
    std::ofstream file(configFile);
    file << "path=" << installPath << "\n";
    file << "ram=" << ramMB << "\n";
    file << "username=" << username << "\n";
    file << "devmode=" << (devMode ? "true" : "false") << "\n";
}

void Config::setup() {
    Utils::clearScreen();
    Utils::printHeader();
    
    std::cout << Utils::Color::YELLOW << "\n⚙️  Настройки\n" << Utils::Color::RESET;
    std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n";
    
    std::cout << "1. Путь установки [" << installPath << "]: ";
    std::string input;
    std::getline(std::cin, input);
    if (!input.empty()) installPath = input;
    
    std::cout << "2. RAM (MB) [" << ramMB << "]: ";
    std::getline(std::cin, input);
    if (!input.empty()) ramMB = std::stoi(input);
    
    std::cout << "3. Username [" << username << "]: ";
    std::getline(std::cin, input);
    if (!input.empty()) username = input;
    
    std::cout << "4. Dev mode (-noverify) [" << (devMode ? "да" : "нет") << "]: ";
    std::getline(std::cin, input);
    if (!input.empty()) devMode = (input == "да" || input == "yes" || input == "y");
    
    save();
    std::cout << Utils::Color::GREEN << "\n✓ Настройки сохранены!\n" << Utils::Color::RESET;
}
