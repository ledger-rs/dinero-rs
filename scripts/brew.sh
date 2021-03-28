# This script outputs th brew formula necessary to install dinero
sha_mac_x86_64=$(shasum -a 256 dinero-mac-x86_64.tar.gz | cut -f 1 -d " ")
sha_mac_aarch64=$(shasum -a 256 dinero-mac-aarch64.tar.gz | cut -f 1 -d " ")

cat << EOF
class Dinero < Formula
    version "0.20.0"
    desc "Command line tool for managing ledger files written in Rust"
    homepage "https://github.com/frosklis/dinero-rs"

    if OS.mac?
        if RUBY_PLATFORM.match(/x86_64/)
            puts "Detected x86_64"
            url "https://github.com/frosklis/dinero-rs/releases/latest/download/dinero-mac-x86_64.tar.gz"
            sha256 "${sha_mac_x86_64}"
        elsif RUBY_PLATFORM.match(/aarch64/)
            puts "Detected aarch64"
            url "https://github.com/frosklis/dinero-rs/releases/latest/download/dinero-mac-aarch64.tar.gz"
            sha256 "${sha_mac_aarch64}"
        end
    end

    def install
        if OS.mac?
            bin.install "dinero"
        else
            puts "Sorry. Only know how to install on a Mac"
        end
    end
end
EOF

  