#include "shared_library.hpp"

#include <windows.h>
#include <stdexcept>
#include <iostream>

class shared_library::loaded_dll {
public:
    HMODULE module;

    explicit loaded_dll(HMODULE module) : module(module) {}

    ~loaded_dll() {
        if (module != nullptr) {
            FreeLibrary(module);
        }
    }
};

shared_library::shared_library() noexcept : instance(nullptr) {}

shared_library::shared_library(const std::string &library_name) {
    HMODULE loaded = LoadLibraryA(library_name.c_str());
    if (loaded == nullptr) {
        std::cout << "Last error: " << GetLastError() << std::endl;
        throw std::runtime_error("Could not load the library");
    }

    this->instance = std::make_shared<shared_library::loaded_dll>(loaded);
}

void *shared_library::getFunctionAddress(const std::string &name) {
    return reinterpret_cast<void *>(GetProcAddress(this->instance->module, name.c_str()));
}