# Aporia Loader

Кроссплатформенный C++ лоадер для Minecraft с Fabric и модами.

## Возможности

- ✅ Поддержка Windows, Linux, macOS
- 🎨 Красивый консольный интерфейс с цветами
- ⚙️ Настройка пути установки, RAM, username
- 📦 Автоматическая загрузка Fabric Loader, API и модов
- 🎮 Выбор модов (Iris, Sodium, Mod Menu и др.)
- 🔧 Dev режим с -noverify

## Сборка

### Требования
- CMake 3.15+
- C++17 компилятор
- libcurl

### Windows
```bash
# Установите libcurl через vcpkg
vcpkg install curl:x64-windows

# Сборка
mkdir build && cd build
cmake .. -DCMAKE_TOOLCHAIN_FILE=[путь к vcpkg]/scripts/buildsystems/vcpkg.cmake
cmake --build . --config Release
```

### Linux/macOS
```bash
# Установите libcurl
# Ubuntu/Debian: sudo apt install libcurl4-openssl-dev
# macOS: brew install curl

# Сборка
mkdir build && cd build
cmake ..
make
```

## Использование

```bash
./aporia-loader
```

### Меню
1. **Запуск** - Загружает все необходимое и запускает Minecraft
2. **Настройки** - Путь, RAM, username, dev режим
3. **Выбор модов** - Включение/отключение модов
4. **Выход**

### Пути по умолчанию
- Windows: `%APPDATA%/apr`
- Linux/macOS: `~/.apr`

## Моды
- Iris Shaders
- Sodium
- Mod Menu
- Sodium Extra
- 3D Skin Layers
- Sound Physics Remastered
- Cloth Config

## Требования
- Java 26+
