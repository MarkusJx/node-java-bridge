#ifndef NODE_JAVA_BRIDGE_UTIL_HPP
#define NODE_JAVA_BRIDGE_UTIL_HPP

#include <string>
#include <vector>
#include <map>
#include <jni.h>

namespace util {
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

    template<class T>
    std::map<std::string, T> map_vector_values_to_map(const std::vector<T> &values) {
        std::map<std::string, T> res;
        for (const T &value : values) {
            res.insert_or_assign(value.name, value);
        }

        return res;
    }

    std::string jni_error_to_string(jint code);

    std::string java_type_to_jni_type(const std::string &to_convert);

    std::string string_replace(std::string val, char to_replace, char replacement);

    jint string_to_java_version(const std::string &to_convert);

    std::string make_java_name_readable(const std::string &to_convert);

    std::string get_java_version_from_jint(jint version);
}

#endif //NODE_JAVA_BRIDGE_UTIL_HPP
