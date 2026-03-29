# Aporia Loader

Кроссплатформенный Rust лоадер для Minecraft с Fabric и модами.

## Возможности

- ✅ Поддержка Windows, Linux, macOS
- 🎨 Красивый GUI интерфейс (egui)
- ⚙️ Настройка пути установки, RAM, username
- 📦 Автоматическая загрузка Fabric Loader, API и модов
- 🎮 Выбор модов (Iris, Sodium, Mod Menu и др.)
- 🔧 Dev режим с -noverify
- ☕ Автозагрузка Java 26

## Сборка

### Требования
- Rust 1.70+
- cargo

### Windows/Linux/macOS
```bash
# Сборка релизной версии
cargo build --release

# Запуск
cargo run --release
```

## Использование

```bash
./aporia-loader
```

### Меню
1. **🚀 Запуск** - Загружает все необходимое и запускает Minecraft
2. **⚙️ Настройки** - Путь, RAM, username, dev режим
3. **📦 Выбор модов** - Включение/отключение модов
4. **📁 Открыть папку** - Открыть директорию лаунчера
5. **❌ Выход**

### Пути по умолчанию
- Windows: `%APPDATA%/apr`
- Linux: `~/.apr`
- macOS: `~/Library/Application Support/apr`

## Моды
- Mod Menu
- 3D Skin Layers
- Sound Physics Remastered
- Cloth Config
- Iris Shaders
- Sodium

## Требования
- Java 26+ (загружается автоматически если не найдена)
