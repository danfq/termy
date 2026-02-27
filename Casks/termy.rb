cask "termy" do
  arch arm: "arm64", intel: "x86_64"

  version "0.1.33"
  sha256 arm:   "fd0b8a25f50cef003eed4d454c2b62c14351994060b6f8faf17f6a7375fe2b65",
         intel: "cb8aecff39d694b782d6f483b21ac56075e7d63fc1aac494ffcab6808d84547c"

  url "https://github.com/lassejlv/termy/releases/download/v#{version}/Termy-v#{version}-macos-#{arch}.dmg"
  name "Termy"
  desc "Minimal GPU-powered terminal written in Rust"
  homepage "https://github.com/lassejlv/termy"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: ">= :big_sur"

  app "Termy.app"
end
