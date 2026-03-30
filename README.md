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

### Локальная сборка

#### Windows
```bash
# Сборка только для Windows
build.bat

# Или вручную
cargo build --release
```

#### Linux/macOS
```bash
# Сборка для текущей платформы
cargo build --release

# Запуск
cargo run --release
```

### Кросс-платформенная сборка

Для сборки под все платформы используйте GitHub Actions:
1. Запушьте код в GitHub
2. Создайте тег: `git tag v0.2.0 && git push origin v0.2.0`
3. GitHub Actions автоматически соберет для Windows, Linux, macOS (x64 и ARM64)
4. Релиз появится в разделе Releases

#### Локальная кросс-компиляция (Linux из Windows)
```bash
# Установите cross и Docker Desktop
cargo install cross

# Убедитесь что Docker запущен
docker --version

# Сборка для Linux
cross build --release --target x86_64-unknown-linux-gnu
```

**Примечание:** macOS сборки возможны только на macOS хосте или через GitHub Actions.

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
