#docker build -t test-im .
#docker run -it test-im
FROM debian:12-slim

RUN apt-get -y update && apt-get -y dist-upgrade && apt-get install -y \
	curl \
	gcc \
	libssl-dev \
	pkg-config \
	ca-certificates && \
	apt-get clean && \
	rm -rf /var/lib/apt/lists/*

ENV SHELL=/usr/bin/bash
#install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile default -c clippy

ENV PATH="/root/.cargo/bin:${PATH}"

# Install zsh + oh-my-zsh (optional)
RUN apt-get update && apt-get install -y zsh git && \
    sh -c "$(curl -fsSL https://raw.githubusercontent.com/robbyrussell/oh-my-zsh/master/tools/install.sh)" && \
    echo "if test -t 1; then\nexec zsh\nfi" >> ~/.bashrc && \
    git clone https://github.com/seebi/dircolors-solarized ~/dircolors-solarized && \
    sed -i.back -e '/ZSH_THEME="robbyrussell"/a eval `dircolors ~/dircolors-solarized/dircolors.256dark`' ~/.zshrc && \
    sed -i.back -e 's/NORMAL 00;38;5;244/NORMAL 00;38;5;230/g' ~/dircolors-solarized/dircolors.256dark && \
    sed -i -e 's/LINK 01;38;5;37/LINK 01;38;5;128/g' ~/dircolors-solarized/dircolors.256dark && \
    sed -i -e 's/EXEC 01;38;5;64/EXEC 01;38;5;230/g' ~/dircolors-solarized/dircolors.256dark && \
    sed -i -e 's/.yml             01;38;5;64/.yml             01;38;5;200/g' ~/dircolors-solarized/dircolors.256dark && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/project