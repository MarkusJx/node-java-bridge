cmake_minimum_required(VERSION 3.16)
project(node_java_bridge)

set(CMAKE_CXX_STANDARD 17)

set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")

# Get the n-api-tools include directory
execute_process(COMMAND node -p "require('@markusjx/n-api-tools').include"
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
        OUTPUT_VARIABLE N_API_TOOLS_DIR)

# Get the java include directories
execute_process(COMMAND node --unhandled-rejections=strict printJavaHome.js
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
        OUTPUT_VARIABLE JAVA_INCLUDE_DIRS
        ERROR_VARIABLE JAVA_HOME_ERROR
        RESULT_VARIABLE FIND_JAVA_HOME_EXIT_CODE)

# Exit with an error if the jdk could not be found
if (NOT FIND_JAVA_HOME_EXIT_CODE EQUAL "0")
    message(FATAL_ERROR "printJavaHome.js failed with error: ${JAVA_HOME_ERROR}")
endif()

# Print the java include directories
message(STATUS "Java include directories: ${JAVA_INCLUDE_DIRS}")

# Include the directories
include_directories(${CMAKE_JS_INC} ${N_API_TOOLS_DIR} ${JAVA_INCLUDE_DIRS} "include" "external")

# Enable n-api exceptions (so we can just throw exceptions
# and n-api will catch them and re-throw them as js exceptions
add_compile_definitions(NAPI_CPP_EXCEPTIONS)

# Set the source files and create the shared library
set(SRC src/module.cpp src/jvm_lib/shared_library.cpp src/jvm_lib/jni_wrapper.cpp src/util.cpp include/util.hpp
        src/node_classes/java.cpp src/node_classes/java_class_proxy.cpp src/node_classes/conversion_helper.cpp
        src/node_classes/java_instance_proxy.cpp src/node_classes/node_jobject_wrapper.cpp)
set(EXTERNAL external/logger.cpp)
add_library(${PROJECT_NAME} SHARED ${SRC} ${EXTERNAL} ${CMAKE_JS_SRC})

# Print the architecture
if ("${CMAKE_SIZEOF_VOID_P}" STREQUAL "4")
    message(STATUS "Building for x32")
elseif ("${CMAKE_SIZEOF_VOID_P}" STREQUAL "8")
    message(STATUS "Building for x64")
endif ()

set_target_properties(${PROJECT_NAME} PROPERTIES PREFIX "" SUFFIX ".node")
target_link_libraries(${PROJECT_NAME} ${CMAKE_JS_LIB})

# Include N-API wrappers
execute_process(COMMAND node -p "require('node-addon-api').include"
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
        OUTPUT_VARIABLE NODE_ADDON_API_DIR)
string(REPLACE "\n" "" NODE_ADDON_API_DIR ${NODE_ADDON_API_DIR})
string(REPLACE "\"" "" NODE_ADDON_API_DIR ${NODE_ADDON_API_DIR})
target_include_directories(${PROJECT_NAME} PRIVATE ${NODE_ADDON_API_DIR})

# define NAPI_VERSION
add_definitions(-DNAPI_VERSION=6)