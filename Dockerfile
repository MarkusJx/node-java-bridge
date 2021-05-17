FROM adoptopenjdk:16-jdk-openj9-focal

# Update
RUN apt-get update
RUN apt-get upgrade -y

# Install build tools
RUN apt-get clean
RUN apt-get update
RUN apt-get install git -y
RUN apt-get install cmake build-essential -y
RUN apt-get install curl -y

# Install node.js
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash -
RUN apt-get install nodejs -y

# Install @markusjx/java globally
RUN npm i -g @markusjx/java@latest
