/**
 * @file config.h
 * @brief Модуль управления конфигурацией лаунчера
 */

#pragma once
#include <string>
#include <fstream>
#include <map>

/**
 * @class Config
 * @brief Класс для работы с настройками лаунчера
 */
class Config {
public:
    std::string installPath;  ///< Путь установки Minecraft
    int ramMB;                ///< Выделенная память в МБ
    std::string username;     ///< Имя пользователя
    bool devMode;             ///< Режим разработчика
    
    /**
     * @brief Конструктор с инициализацией значений по умолчанию
     */
    Config();
    
    /**
     * @brief Загружает конфигурацию из файла
     */
    void load();
    
    /**
     * @brief Сохраняет конфигурацию в файл
     */
    void save();
    
    /**
     * @brief Интерактивная настройка конфигурации
     */
    void setup();
    
private:
    std::string configFile;                    ///< Путь к файлу конфигурации
    std::map<std::string, std::string> data;   ///< Данные конфигурации
    
    /**
     * @brief Удаляет пробелы в начале и конце строки
     * @param str Исходная строка
     * @return Обрезанная строка
     */
    std::string trim(const std::string& str);
};
