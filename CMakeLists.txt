cmake_minimum_required(VERSION 3.19)
project(node_java_bridge)

set(CMAKE_CXX_STANDARD 20)

include_directories(${CMAKE_JS_INC} ${N_API_TOOLS_DIR})
add_compile_definitions(NAPI_CPP_EXCEPTIONS)

execute_process(COMMAND node -p "require('@markusjx/n-api-tools').include"
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
        OUTPUT_VARIABLE N_API_TOOLS_DIR)

# Include the directory
include_directories(${N_API_TOOLS_DIR})
include_directories("C:\\Program Files\\AdoptOpenJDK\\jdk-11.0.10.9-hotspot\\include")
include_directories("C:\\Program Files\\AdoptOpenJDK\\jdk-11.0.10.9-hotspot\\include\\win32")
include_directories(include)

set(SRC src/module.cpp src/shared_library.cpp src/shared_library.hpp src/jni_wrapper.hpp src/jni_wrapper.cpp)
add_executable(node_java_bridge ${SRC})
#add_library(node_java_bridge ${SRC})

#set_target_properties(${PROJECT_NAME} PROPERTIES PREFIX "" SUFFIX ".node")
#target_link_libraries(${PROJECT_NAME} ${CMAKE_JS_LIB})

# Include N-API wrappers
#execute_process(COMMAND node -p "require('node-addon-api').include"
#        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
#        OUTPUT_VARIABLE NODE_ADDON_API_DIR
#        )
#string(REPLACE "\n" "" NODE_ADDON_API_DIR ${NODE_ADDON_API_DIR})
#string(REPLACE "\"" "" NODE_ADDON_API_DIR ${NODE_ADDON_API_DIR})
#target_include_directories(${PROJECT_NAME} PRIVATE ${NODE_ADDON_API_DIR})

# define NPI_VERSION
add_definitions(-DNAPI_VERSION=4)