cask "termy" do
  arch arm: "arm64", intel: "x86_64"

  version "0.1.41"
  sha256 arm:   "3e447c263f149eff88b5046055c7f99a207494654a8b1e3d480cbeed71df9ded",
         intel: "911cec36951306d30a53c03b40941826ee2a9b3a83a3586f09c6bec2cef3c719"

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
