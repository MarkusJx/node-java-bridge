#ifndef NODE_JAVA_BRIDGE_UTIL_HPP
#define NODE_JAVA_BRIDGE_UTIL_HPP

#include <string>
#include <vector>
#include <map>
#include <jni.h>

/**
 * Utilities
 */
namespace util {
    /**
     * Convert a vector to a map.
     * T must have a member called 'name' of type string.
     *
     * @tparam T the type to map
     * @param values the values to map
     * @return the map with T::name as a key
     */
    template<class T>
    std::map<std::string, std::vector<T>> map_vector_to_map(const std::vector<T> &values) {
        std::map<std::string, std::vector<T>> res;
        for (const T &value : values) {
            if (res.find(value.name) != res.end()) {
                res.at(value.name).push_back(value);
            } else {
                res.insert_or_assign(value.name, std::vector<T>({value}));
            }
        }

        return res;
    }

    /**
     * Convert a vector to a map
     * T must have a member called 'name' of type string.
     *
     * @tparam T the type to map
     * @param values the values to map
     * @return the map with T::name as a key
     */
    template<class T>
    std::map<std::string, T> map_vector_values_to_map(const std::vector<T> &values) {
        std::map<std::string, T> res;
        for (const T &value : values) {
            res.insert_or_assign(value.name, value);
        }

        return res;
    }

    /**
     * Convert a jni error to a string
     *
     * @param code the error code to convert
     * @return the converted error code
     */
    std::string jni_error_to_string(jint code);

    /**
     * Convert a java type to jni type
     *
     * @param to_convert the value to convert
     * @return the converted type
     */
    std::string java_type_to_jni_type(const std::string &to_convert);

    /**
     * Replace characters in a string
     *
     * @param val the string to replace the values in
     * @param to_replace the char to replace
     * @param replacement the replacement
     * @return the edited string
     */
    std::string string_replace(std::string val, char to_replace, char replacement);

    /**
     * Convert a version string to a java version identifier
     *
     * @param to_convert the version string to convert
     * @return the converted version identifier
     */
    jint string_to_java_version(const std::string &to_convert);

    /**
     * Make a jni signature readable
     *
     * @param to_convert the signature to convert
     * @return the converted signature
     */
    std::string make_java_name_readable(const std::string &to_convert);

    /**
     * Get a java version string from a version identifier
     *
     * @param version the version to convert
     * @return the version string
     */
    std::string get_java_version_from_jint(jint version);

    /**
     * Check if a string has an ending.
     * Source: https://stackoverflow.com/a/874160
     *
     * @param fullString the string to check
     * @param ending the ending to check
     * @return true if fullString ends with ending
     */
    bool hasEnding(std::string const &fullString, std::string const &ending);

    /**
     * Check if a signature equals to a java primitive type,
     * e.g. int, long, float, double...
     *
     * @param signature the signature to check
     * @return true if it is a primitive type
     */
    bool isPrimitive(const std::string &signature);
}

#endif //NODE_JAVA_BRIDGE_UTIL_HPP
