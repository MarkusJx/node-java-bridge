#include <sstream>

#include "jvm_lib/java_exception.hpp"

using namespace jni;

java_exception::java_exception(const std::vector<std::string> &causes, const std::vector<std::string> &frames)
        : std::exception(), message(generateErrorMessage(causes, frames)) {}

const char *java_exception::what() const noexcept {
    return message.c_str();
}

std::string java_exception::generateErrorMessage(const std::vector<std::string> &causes,
                                                 const std::vector<std::string> &frames) {
    // NOTE: Can't use string streams, the module will crash
    try {
        std::stringstream res;
        for (size_t i = 0; i < causes.size(); i++) {
            if (i == 0) {
                res << causes[i];

                if (!frames.empty()) {
                    res << '\n';
                }

                for (const std::string &frame : frames) {
                    res << '\t' << "at " << frame << std::endl;
                }
            } else {
                res << "Caused by: " << causes[i];
            }

            if ((i + 1) < causes.size()) {
                res << '\n';
            }
        }

        return res.str();
    } catch (...) {
        return "Could not get the error message";
    }
}