const ConvertLib = artifacts.require("ConvertLib");
const GitToken = artifacts.require("GitToken");

module.exports = function (deployer) {
  deployer.deploy(ConvertLib);
  deployer.link(ConvertLib, GitToken);
  deployer.deploy(GitToken, 1000);
};
