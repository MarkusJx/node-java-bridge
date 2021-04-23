#ifndef NODE_JAVA_BRIDGE_SHARED_LIBRARY_HPP
#define NODE_JAVA_BRIDGE_SHARED_LIBRARY_HPP

#include <memory>
#include <string>
#include <functional>

class shared_library {
public:
    shared_library() noexcept;

    explicit shared_library(const std::string &library_name);

    void *getFunctionAddress(const std::string &name);

    template<class T>
    std::function<T> getFunction(const std::string &name) {
        return std::function<T>(reinterpret_cast<T*>(getFunctionAddress(name)));
    }

private:
    class loaded_dll;

    std::shared_ptr<loaded_dll> instance;
};

#endif //NODE_JAVA_BRIDGE_SHARED_LIBRARY_HPP
