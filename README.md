# MCSL Compiler

**Minecraft Structured Language** — компилятор языка программирования для создания датапаков Minecraft 1.21.1.

## Возможности

- **Упрощённый синтаксис** — пиши код проще, чем ванильные команды
- **Полная поддержка Minecraft 1.21.1** — все команды доступны
- **Спец-аргументы** — `@` для сущностей и координат, `%` для локальных координат
- **Функции и теги** — `$load` и `$tick` для автоматической регистрации
- **Условные блоки** — `if (...) { ... }` вместо `execute if ... run ...`
- **Алиасы команд** — используйте короткие имена (`#tp`, `#gm`, `#eff` и др.)

## Быстрый старт

```mcsl
// Tick функция - выполняется каждый тик
$tick func main {
    #tp @p @~ @~ @~
    #say "Hello World"
}

// Load функция - выполняется при загрузке
$load func init {
    #gamerule doDaylightCycle false
    #tellraw(target: @a, message: ["Welcome!", "gold", true, false])
}

// Обычная функция
func give_diamond {
    #give @p diamond 64
}
```

## Синтаксис

### Специальные префиксы

| Префикс | Назначение | Пример |
|---------|------------|--------|
| `@` | Сущности и относительные координаты | `@a`, `@p`, `@e`, `@~`, `@~5` |
| `%` | Префикс для локальных координат | `%` |
| `%^` | Локальные координаты | `%^`, `%^5` |
| `#` | Команды | `#say`, `#give`, `#tp` |
| `$` | Теги функций | `$load`, `$tick` |
| `*` | Wildcard (для рецептов и т.д.) | `#rec take @a *` |

### Функции

```mcsl
// Функция без тега
func my_function {
    #say "Hello"
}

// Tick функция (выполняется каждый тик)
$tick func game_loop {
    #tp @a @~ @~ @~
}

// Load функция (выполняется при загрузке)
$load func init {
    #gamerule keepInventory true
}
```

### Команды

```mcsl
// Простой синтаксис
#say "Hello World"
#give @p diamond 10
#tp @a @~ @~5 @~

// С именованными аргументами
#tellraw(target: @a, message: ["Hello", "red", true, false])
#teleport(entity: @p, coords: [@~ @~ @~])
#fill(from: [@~ @~ @~], to: [@~10 @~10 @~10], block: stone)

// С селекторами
#kill @e[type=zombie]
#effect give @a speed 60 1
#summon creeper @~ @~ @~
```

### Координаты

```mcsl
// Абсолютные
#tp @p 100 64 200

// Относительные (@~ или ~)
#tp @p @~ @~ @~
#tp @p @~5 @~ @~10

// Локальные (%^)
#tp @p %^ %^ %^
```

### Условные блоки

```mcsl
func check_players {
    if (@a) {
        #say "Players online"
    }
}
```

### Массивы

```mcsl
// Координаты в массивах
#fill @~ @~ @~ @~10 @~10 @~10 stone

// Tellraw сообщения
#tellraw(target: @a, message: ["Hello", "gold", true, false])
```

## Поддерживаемые команды

### Основные (Core)

| Команда | Алиасы | Описание |
|---------|--------|----------|
| `#function` | `#fn`, `#run` | Вызов функции |
| `#execute` | `#exec` | Выполнение с контекстом |
| `#data` | — | Работа с NBT данными |
| `#item` | — | Работа с предметами |
| `#scoreboard` | `#sb`, `#cscoreboard`, `#csb` | Счётчики |
| `#tag` | — | Теги сущностей |
| `#team` | — | Команды |
| `#schedule` | `#sched` | Отложенное выполнение |
| `#return` | — | Возврат значения |
| `#random` | `#rng` | Генерация случайных чисел |
| `#tick` | — | Управление тиками |
| `#reload` | — | Перезагрузка датапаков |

### Сущности и перемещение

| Команда | Алиасы | Описание |
|---------|--------|----------|
| `#summon` | `#spawn` | Призыв сущности |
| `#damage` | `#dmg` | Нанесение урона |
| `#kill` | — | Убийство сущности |
| `#teleport` | `#tp`, `#tele` | Телепортация |
| `#ride` | `#mount` | Поседлать сущность |
| `#rotate` | `#rot` | Поворот сущности |

### Блоки и мир

| Команда | Алиасы | Описание |
|---------|--------|----------|
| `#setblock` | `#setb` | Установка блока |
| `#fill` | — | Заполнение области |
| `#clone` | — | Копирование области |
| `#fillbiome` | `#biome` | Заполнение биомами |
| `#place` | — | Размещение структур |
| `#forceload` | `#load` | Принудительная загрузка чанков |

### Предметы и инвентарь

| Команда | Алиасы | Описание |
|---------|--------|----------|
| `#give` | `#g` | Выдача предмета |
| `#clear` | `#clr` | Очистка инвентаря |
| `#loot` | — | Выдача лута |
| `#use` | — | Использование предмета |

### Состояние игрока

| Команда | Алиасы | Описание |
|---------|--------|----------|
| `#effect` | `#eff`, `#potion` | Эффекты |
| `#enchant` | `#ench` | Зачарование |
| `#experience` | `#xp`, `#exp` | Опыт |
| `#gamemode` | `#gm` | Режим игры |
| `#advancement` | `#adv`, `#achievement` | Достижения |
| `#attribute` | `#attr` | Атрибуты |
| `#recipe` | `#rec` | Рецепты |
| `#inputpermission` | `#inputperm` | Разрешения ввода |

### Отображение и звук

| Команда | Алиасы | Описание |
|---------|--------|----------|
| `#title` | — | Заголовки |
| `#tellraw` | `#message`, `#msg` | Сообщения |
| `#bossbar` | `#boss` | Босс-бары |
| `#particle` | `#part`, `#particles` | Частицы |
| `#playsound` | `#sound`, `#play` | Воспроизведение звука |
| `#stopsound` | `#stops` | Остановка звука |
| `#jfr` | — | Профилирование JFR |

### Мир и окружение

| Команда | Алиасы | Описание |
|---------|--------|----------|
| `#time` | — | Время |
| `#weather` | `#wx` | Погода |
| `#gamerule` | `#rule`, `#gr` | Правила игры |
| `#difficulty` | `#diff` | Сложность |
| `#spawnpoint` | `#spawn` | Точка возрождения |
| `#setworldspawn` | `#worldspawn` | Точка возрождения мира |
| `#worldborder` | `#border` | Граница мира |

### Утилиты

| Команда | Алиасы | Описание |
|---------|--------|----------|
| `#say` | — | Сообщение в чат |
| `#tell` | `#w`, `#whisper`, `#msg` | Личное сообщение |
| `#me` | — | Действие от имени |
| `#help` | `#h` | Справка |
| `#datapack` | `#pack` | Управление датапаками |
| `#seed` | — | Сид мира |
| `#locate` | `#loc` | Поиск структур |
| `#kick` | — | Кик игрока |
| `#list` | — | Список игроков |
| `#perf` | — | Профилирование |
| `#save` | — | Сохранение мира |
| `#stop` | — | Остановка сервера |
| `#whitelist` | — | Белый список |
| `#ban` | — | Бан |
| `#pardon` | — | Разбан |
| `#op` | — | Выдать оператора |
| `#deop` | — | Забрать оператора |

### Отладка

| Команда | Алиасы | Описание |
|---------|--------|----------|
| `#debug` | — | Отладка |
| `#publish` | — | Открыть в LAN |
| `#spectate` | — | Режим наблюдателя |
| `#jigsaw` | — | Джигсоу блоки |
| `#chunk` | — | Управление чанками |

## Сборка

```bash
# Отладочная сборка
cargo build

# Релизная сборка
cargo build --release
```

## Использование

```bash
# Базовая компиляция
./target/release/mcsl_compiler -i input.mcsl -o output

# С опциями
./target/release/mcsl_compiler \
    --input my_datapack.mcsl \
    --output build \
    --namespace mypack \
    --verbose
```

### Опции командной строки

| Опция | Краткая | Описание | По умолчанию |
|-------|---------|----------|--------------|
| `--input` | `-i` | Входной MCSL файл | — |
| `--output` | `-o` | Выходная директория | `output` |
| `--namespace` | `-n` | Namespace для датапака | `mcsldatapack` |
| `--verbose` | `-v` | Подробный вывод | `false` |

## Структура проекта

```
mcsl_compiler/
├── src/
│   ├── main.rs      # CLI и точка входа
│   ├── ast.rs       # AST определения
│   ├── lexer.rs     # Лексический анализ
│   ├── parser.rs    # Синтаксический разбор
│   ├── codegen.rs   # Генерация кода
│   └── compiler.rs  # Основная логика компиляции
├── Cargo.toml
├── README.md
└── test_*.mcsl      # Тестовые файлы
```

## Структура выходного датапака

```
output/
├── pack.mcmeta                          # Метаданные датапака
└── data/
    └── <namespace>/
        ├── functions/
        │   ├── main.mcfunction          # Основная функция
        │   ├── tick.mcfunction          # Tick функция
        │   └── tags/
        │       └── function/
        │           ├── tick.json        # Tick тег
        │           └── load.json        # Load тег
        └── minecraft/
            └── tags/
                └── functions/
                    ├── tick.json
                    └── load.json
```

## Примеры

### Автоматическая ферма

```mcsl
$tick func farm_controller {
    #execute if entity @e[type=cow] run #say "Cows detected"
}

$load func setup {
    #gamerule doMobSpawning true
    #gamerule randomTickSpeed 3
    #say "Farm datapack loaded!"
}
```

### Система китов

```mcsl
func give_starter_kit {
    #give @p iron_sword 1
    #give @p bread 32
    #give @p leather_chestplate 1
    #effect give @p health_boost 60 0
}

$load func register {
    #say "Starter kit system ready"
}
```

### Телепортация по команде

```mcsl
$tick func teleport_system {
    if (@a[tag=warp]) {
        #tp @a[tag=warp] 0 100 0
        #tag @a remove warp
    }
}
```

## Планы развития

- [x] Поддержка всех команд Minecraft 1.21.1
- [ ] Расширенная обработка ошибок с указанием строк
- [ ] Поддержка переменных и вычислений
- [ ] Макросы и шаблоны
- [ ] Интеграция с IDE (LSP)
- [ ] Поддержка предикатов и лута

## Лицензия

MIT
