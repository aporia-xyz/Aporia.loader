#pragma once
#include <string>
#include <vector>
#include <fstream>
#include <filesystem>

namespace fs = std::filesystem;

class ResourceExtractor {
public:
    static std::vector<unsigned char> base64_decode(const std::string& encoded) {
        static const std::string base64_chars = 
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            "abcdefghijklmnopqrstuvwxyz"
            "0123456789+/";
        
        std::vector<unsigned char> decoded;
        std::vector<int> T(256, -1);
        for (int i = 0; i < 64; i++) T[base64_chars[i]] = i;
        
        int val = 0, valb = -8;
        for (unsigned char c : encoded) {
            if (T[c] == -1) break;
            val = (val << 6) + T[c];
            valb += 6;
            if (valb >= 0) {
                decoded.push_back(char((val >> valb) & 0xFF));
                valb -= 8;
            }
        }
        return decoded;
    }
    
    static bool extractResource(const std::string& base64_data, const std::string& output_path) {
        auto decoded = base64_decode(base64_data);
        
        fs::create_directories(fs::path(output_path).parent_path());
        
        std::ofstream file(output_path, std::ios::binary);
        if (!file.is_open()) return false;
        
        file.write(reinterpret_cast<const char*>(decoded.data()), decoded.size());
        return true;
    }
};
