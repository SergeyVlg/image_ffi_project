# image_ffi_project
Учебный проект на Rust по работе с динамически линкуемыми библиотеками в рантайме.

# Состав проекта
Проект содержит CLI утилиту (бинарный крейт image_processor) для обработки входного изображения с помощью двух плагинов - отражения (mirror_plugin) и/или размытия (blur_plugin).
Плагины и CLI утилита включены в общий workspace.

# Порядок работы
Для работы необходимо сбилдить необходимые проекты плагинов через cargo build (или весь workspace). После этого можно использовать CLI для работы с изображением.
Запускать её необходимо со следующими параметрами:

```plain
cargo run -- --input <путь к исходному png файлу> --output <путь к обработанному png файлу> --plugin <тип плагина - mirror или blur> --params <путь к файлу с параметрами> --plugin-path <путь к папке с dll плагина без указания файла>
```
Например:

```plain
cargo run -- --input test.png --output processed.png --plugin blur --params blur_params.json --plugin-path ..\target\debug\
```

Примеры настроек плагинов хранятся в файлах blur_params.json и mirror_params.json внутри их крейтов.