cmake_minimum_required(VERSION 3.15)
project(AporiaLoader)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(CURL REQUIRED)

set(SOURCES
    src/main.cpp
    src/downloader.cpp
    src/config.cpp
)

if(WIN32)
    list(APPEND SOURCES src/app.rc)
endif()

add_executable(aporia-loader ${SOURCES})

target_include_directories(aporia-loader PRIVATE ${CURL_INCLUDE_DIRS})
target_link_libraries(aporia-loader PRIVATE ${CURL_LIBRARIES})

if(WIN32)
    target_compile_definitions(aporia-loader PRIVATE PLATFORM_WINDOWS)
elseif(APPLE)
    target_compile_definitions(aporia-loader PRIVATE PLATFORM_MACOS)
else()
    target_compile_definitions(aporia-loader PRIVATE PLATFORM_LINUX)
endif()
