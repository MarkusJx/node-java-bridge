#ifndef NODE_JAVA_BRIDGE_JAVA_EXCEPTION_HPP
#define NODE_JAVA_BRIDGE_JAVA_EXCEPTION_HPP

#include <string>
#include <vector>

namespace jni {
    /**
     * A generic java exception
     */
    class java_exception : public std::exception {
    public:
        /**
         * A java exception
         *
         * @param causes the list of exception that caused this exception
         * @param frames the stack frames
         */
        java_exception(const std::vector<std::string> &causes, const std::vector<std::string> &frames);

        [[nodiscard]] const char *what() const override;

    private:
        /**
         * Generate the error message
         *
         * @param causes the list of exception that caused this exception
         * @param frames the stack frames
         * @return the generated error message containing all information
         */
        static std::string generateErrorMessage(const std::vector<std::string> &causes,
                                                const std::vector<std::string> &frames);

        // The error message
        std::string message;
    };
}

#endif //NODE_JAVA_BRIDGE_JAVA_EXCEPTION_HPP
