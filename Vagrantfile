Vagrant.configure(2) do |config|
  config.vm.box = "ubuntu/trusty64"
  config.vm.network :private_network, ip: "192.168.83.42"
  config.ssh.forward_agent = true
  config.vm.synced_folder "./", "/intecture", type: "nfs"
  config.vm.provision "shell", inline: "apt-get update && apt-get -y install git build-essential pkg-config libtool autoconf autogen && \
  	curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- -y && \
  	cp -R /intecture /tmp/ && cd /tmp/intecture && \
  	./.travis_install.sh && \
  	export LD_LIBRARY_PATH=/lib:/usr/lib:/usr/local/lib && \
  	cargo build --verbose && cargo test --verbose"
end
