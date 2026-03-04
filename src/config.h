#pragma once
#include <string>
#include <fstream>
#include <map>

class Config {
public:
    std::string installPath;
    int ramMB;
    std::string username;
    bool devMode;
    
    Config();
    void load();
    void save();
    void setup();
    
private:
    std::string configFile;
    std::map<std::string, std::string> data;
    
    std::string trim(const std::string& str);
};
