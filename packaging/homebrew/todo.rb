class Todo < Formula
  desc "High-performance cross-platform Todo app with Web, TUI, CLI, and GUI"
  homepage "https://github.com/itsvrushabh/todo"
  version "0.1.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/itsvrushabh/todo/releases/download/v#{version}/taskmaster-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/itsvrushabh/todo/releases/download/v#{version}/taskmaster-macos-x86_64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/itsvrushabh/todo/releases/download/v#{version}/taskmaster-linux-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/itsvrushabh/todo/releases/download/v#{version}/taskmaster-linux-amd64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    bin.install "todo"
  end

  test do
    system "#{bin}/todo", "--version"
  end
end
