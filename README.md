# github-upvotes

A simple project to get a list of the most upvoted issues in a GitHub
repository.

## Set up

Create a `.env` file in the root of the project with the following content:

```sh
GITHUB_PAT=your_github_personal_access_token
USER_AGENT=your_github_username
```

## Running the project

To run the project, execute the following command:

```sh
cargo run -- <owner> <repository> <limit>
```
