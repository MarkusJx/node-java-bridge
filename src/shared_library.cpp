#if defined(WIN32) || defined(_WIN32) || defined(__WIN32) && !defined(__CYGWIN__)
#   define SHARED_LIBRARY_WINDOWS
#elif defined(__LINUX__) || defined(__APPLE__) || defined (__CYGWIN__) || defined(__linux__) || defined(__FreeBSD__) || \
        defined(unix) || defined(__unix) || defined(__unix__)
#   undef SHARED_LIBRARY_WINDOWS
#endif

#include "shared_library.hpp"

#include <stdexcept>

#ifdef SHARED_LIBRARY_WINDOWS

#   include <windows.h>

#else
#   include <dlfcn.h>
#endif //SHARED_LIBRARY_WINDOWS

#ifdef SHARED_LIBRARY_WINDOWS

// Source: https://stackoverflow.com/a/17387176
std::string GetLastErrorAsString() {
    //Get the error message ID, if any.
    DWORD errorMessageID = ::GetLastError();
    if (errorMessageID == 0) {
        return std::string(); //No error message has been recorded
    }

    LPSTR messageBuffer = nullptr;

    //Ask Win32 to give us the string version of that message ID.
    //The parameters we pass in, tell Win32 to create the buffer that holds the message for us (because we don't yet know how long the message string will be).
    size_t size = FormatMessageA(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS, nullptr,
            errorMessageID, MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), (LPSTR) &messageBuffer, 0, nullptr);

    // If the message ends with a new line
    // (+ a carriage return ['\r'], this is still windows) remove that
    if (size >= 2 && messageBuffer[size - 1] == '\n') {
        size -= 2;
    }

    //Copy the error message into a std::string.
    std::string message(messageBuffer, size);

    //Free the Win32's string's buffer.
    LocalFree(messageBuffer);

    return message;
}

#endif //SHARED_LIBRARY_WINDOWS


class shared_library::loaded_dll {
public:
#ifdef SHARED_LIBRARY_WINDOWS
    HMODULE module;

    explicit loaded_dll(HMODULE module) : module(module) {}

#else
    void *module;

    explicit loaded_dll(void *module) : module(module) {}
#endif //SHARED_LIBRARY_WINDOWS

    ~loaded_dll() {
        if (module != nullptr) {
#ifdef SHARED_LIBRARY_WINDOWS
            FreeLibrary(module);
#else
            dlclose(module);
#endif //SHARED_LIBRARY_WINDOWS
        }
    }
};

shared_library::shared_library() noexcept: instance(nullptr) {}

shared_library::shared_library(const std::string &library_name) {
#ifdef SHARED_LIBRARY_WINDOWS
    SetLastError(0);
    HMODULE loaded = LoadLibraryA(library_name.c_str());
    if (loaded == nullptr) {
        throw std::runtime_error("Could not load the library. Reason: " + GetLastErrorAsString());
    }
#else
    void *loaded = dlopen(library_name.c_str(), RTLD_LAZY);
    if (loaded == nullptr) {
        throw std::runtime_error("Could not load the library. Reason: " + std::string(dlerror()));
    }
#endif //SHARED_LIBRARY_WINDOWS

    this->instance = std::make_shared<shared_library::loaded_dll>(loaded);
}

void *shared_library::getFunctionAddress(const std::string &name) {
#ifdef SHARED_LIBRARY_WINDOWS
    void *symbol = reinterpret_cast<void *>(GetProcAddress(this->instance->module, name.c_str()));
    if (symbol == nullptr) {
        throw std::runtime_error("Could not resolve the function. Reason: " + GetLastErrorAsString());
    } else {
        return symbol;
    }
#else
    void *symbol = dlsym(this->instance->module, name.c_str());
    if (symbol == nullptr) {
        throw std::runtime_error("Could not resolve the function. Reason: " + std::string(dlerror()));
    } else {
        return symbol;
    }
#endif //SHARED_LIBRARY_WINDOWS
}