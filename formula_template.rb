class TPAWSCli < Formula
  desc "{{description}}"
  homepage "{{homepage}}"
  url "{{repo}}/releases/latest/download/{{bin}}.tar.gz"
  sha256 "{{shasum}}"
  version "{{version}}"

  def install 
    bin.install tpaws
  end
end
