#ifndef NODE_JAVA_BRIDGE_SHARED_LIBRARY_HPP
#define NODE_JAVA_BRIDGE_SHARED_LIBRARY_HPP

#include <memory>
#include <string>
#include <functional>

/**
 * A shared library.
 * This class is able to dynamically
 * load a shared library and get functions from it.
 */
class shared_library {
public:
    /**
     * Create a invalid shared library
     */
    shared_library() noexcept;

    /**
     * Create a shared library
     *
     * @param library_name the path to the library to load
     */
    explicit shared_library(const std::string &library_name);

    /**
     * Get a function address
     *
     * @param name the name of the function to resolve
     * @return the function's address
     */
    void *getFunctionAddress(const std::string &name);

    /**
     * Get a function as a std::function
     *
     * @tparam T the type of the function
     * @param name the name of the function to resolve
     * @return the function
     */
    template<class T>
    std::function<T> getFunction(const std::string &name) {
        return std::function<T>(reinterpret_cast<T*>(getFunctionAddress(name)));
    }

private:
    class loaded_dll;

    // The loaded shared library instance
    std::shared_ptr<loaded_dll> instance;
};

#endif //NODE_JAVA_BRIDGE_SHARED_LIBRARY_HPP
