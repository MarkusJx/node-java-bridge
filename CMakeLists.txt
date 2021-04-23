cmake_minimum_required(VERSION 3.16)
project(node_java_bridge)

set(CMAKE_CXX_STANDARD 17)

execute_process(COMMAND node -p "require('@markusjx/n-api-tools').include"
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
        OUTPUT_VARIABLE N_API_TOOLS_DIR)

# Include the directory
include_directories(${CMAKE_JS_INC} ${N_API_TOOLS_DIR})
add_compile_definitions(NAPI_CPP_EXCEPTIONS)
include_directories("C:\\Program Files\\AdoptOpenJDK\\jdk-11.0.10.9-hotspot\\include")
include_directories("C:\\Program Files\\AdoptOpenJDK\\jdk-11.0.10.9-hotspot\\include\\win32")
include_directories(include)

set(SRC src/module.cpp src/jvm_lib/shared_library.cpp src/jvm_lib/jni_wrapper.cpp src/util.cpp include/util.hpp
        src/node_classes/java.cpp src/node_classes/java_class_proxy.cpp src/node_classes/conversion_helper.cpp)
#add_executable(node_java_bridge_exe ${SRC})
add_library(${PROJECT_NAME} SHARED ${SRC} ${CMAKE_JS_SRC})

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
        OUTPUT_VARIABLE NODE_ADDON_API_DIR
        )
string(REPLACE "\n" "" NODE_ADDON_API_DIR ${NODE_ADDON_API_DIR})
string(REPLACE "\"" "" NODE_ADDON_API_DIR ${NODE_ADDON_API_DIR})
target_include_directories(${PROJECT_NAME} PRIVATE ${NODE_ADDON_API_DIR})

# define NPI_VERSION
add_definitions(-DNAPI_VERSION=6)