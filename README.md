# Omave dapp

Open source incentivizing on the Solana blockchain

## The open contribution token (OCT)

The open contribution token is meant to incentivise all developers to contribute and to participate in the open source community. This will happen through easily donating to projects that developers often use. The project owners can then stake or reward individual contributers that submit issues, bugs, reviews pull request and even commit code. The goal is to create a thriving ecosystem around OCT.

The motivation behind creating the token is based on the hypothesis that people that contributes to open source are mainly invested in the project in some form through their current employer. This could either be because they are developing it within the organization, like say React. Or that they are using an open source project in one of their application, like a web server abstraction in any language. In these cases the incentivization happens through the employer. However, when the workday is over the motivation for contribution might fade away and the lust for recreation is greater. If one of the recreational activities are development then it might be more tempting to develop something for yourself rather than contributing to a project. The consequence of this can then become creatives working on their own radical project but with few people motivated to contribute the perseverance can be put to the test.

This is the problem to be solved: Motivate people to improve software together and award contributers.

## FAQ

### Why can't the project be built on a the bank system that we all are used to?

There are several advantages to building the Omave project on top of a high throughput blockchain. Some of these are

- Asssumption of international developers operating across different home fiats. It is better to have one currency for contribution
- One key feature has to be scalable throughput with low fees. This is easily acessible with the modern proof of stake blockchains.
- Little setup on the user side. Just create a wallet and link it to the github profile.

## The blockchain

One of the primary requirements of the OCT is to be fast and cheap as the goal is to get new production quality code on the main branch as fast as possible. Security is of course important but the bulk of the OCT transactions are expected to be small. Additionally, for the Omave project to survive it is important to provide a greate developer experience in a safe language. Therefore:

The project is built on the Solana blockchain.

## Alpha version

The first version is to tackle the challenge of getting contributers to submit quality pull request reviews as fast as possible.

The key features to make this efficient are

- Allow the pull request creator to select the maximum number of OCT as a reward R_max for the pull request
- The review value estimator will score the review on a scale of 0 to 1. Therefore the final award will be R in [0,R_max]. The case of 0 awarded OCT will in theory happen when a contributer makes a review without any kind of contructive comments. This often happens with the so called LGTM (looks good to me) review comments.
- If multiple people contributes the total total reward R will be divided amongst the participants.
