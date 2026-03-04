#pragma once
#include <string>
#include <iostream>
#include <vector>
#include <fstream>

#ifdef _WIN32
    #include <windows.h>
    #include <conio.h>
#else
    #include <unistd.h>
    #include <termios.h>
#endif

namespace Utils {
    // ANSI color codes
    namespace Color {
        const std::string RESET = "\033[0m";
        const std::string RED = "\033[31m";
        const std::string GREEN = "\033[32m";
        const std::string YELLOW = "\033[33m";
        const std::string BLUE = "\033[34m";
        const std::string MAGENTA = "\033[35m";
        const std::string CYAN = "\033[36m";
        const std::string WHITE = "\033[37m";
        const std::string BOLD = "\033[1m";
    }

    inline void enableColors() {
#ifdef _WIN32
        SetConsoleOutputCP(CP_UTF8);
        SetConsoleCP(CP_UTF8);
        HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
        DWORD dwMode = 0;
        GetConsoleMode(hOut, &dwMode);
        SetConsoleMode(hOut, dwMode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
#endif
    }

    inline void clearScreen() {
#ifdef _WIN32
        system("cls");
#else
        system("clear");
#endif
    }

    inline std::string getDefaultPath() {
#ifdef _WIN32
        const char* appdata = getenv("APPDATA");
        return std::string(appdata ? appdata : "") + "\\apr";
#else
        const char* home = getenv("HOME");
        return std::string(home ? home : "") + "/.apr";
#endif
    }

    inline std::string findJava() {
#ifdef _WIN32
        // Проверяем стандартные пути Java
        std::vector<std::string> javaPaths = {
            "C:\\Program Files\\Java\\jdk-26\\bin\\java.exe",
            "C:\\Program Files\\Java\\jdk-25.0.2\\bin\\java.exe",
            "C:\\Program Files\\Java\\jdk-21.0.10\\bin\\java.exe",
            "C:\\Program Files\\Java\\latest\\bin\\java.exe"
        };
        
        for (const auto& path : javaPaths) {
            std::ifstream f(path);
            if (f.good()) {
                f.close();
                return "\"" + path + "\"";
            }
        }
        
        // Если не нашли, пробуем через PATH
        return "java";
#else
        return "java";
#endif
    }

    inline void printHeader() {
        std::cout << Color::CYAN << Color::BOLD;
        std::cout << R"(
                 █████╗ ██████╗  ██████╗ ██████╗ ██╗ █████╗ 
                ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██║██╔══██╗
                ███████║██████╔╝██║   ██║██████╔╝██║███████║
                ██╔══██║██╔═══╝ ██║   ██║██╔══██╗██║██╔══██║
                ██║  ██║██║     ╚██████╔╝██║  ██║██║██║  ██║
                ╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝╚═╝╚═╝  ╚═╝
                                                    
                            🎮 Loader v0.1 🎮
)" << Color::RESET << "\n";
    }

    inline int getKeyPress() {
#ifdef _WIN32
        int ch = _getch();
        if (ch == 0 || ch == 224) {
            ch = _getch();
            switch(ch) {
                case 72: return 'w'; // Up arrow
                case 80: return 's'; // Down arrow
            }
        }
        return ch;
#else
        struct termios oldt, newt;
        tcgetattr(STDIN_FILENO, &oldt);
        newt = oldt;
        newt.c_lflag &= ~(ICANON | ECHO);
        tcsetattr(STDIN_FILENO, TCSANOW, &newt);
        int ch = getchar();
        tcsetattr(STDIN_FILENO, TCSANOW, &oldt);
        return ch;
#endif
    }
}
