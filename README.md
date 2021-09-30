# settings-loader-rs
Opinionated configuration settings load mechanism for Rust applications

# configuration
Configuration for both the <code>server</code> and <code>loader</code> is loaded from a combination
of potential sources. (This mechanism was copied from some of my other personal projects, and will
be consolidated into a separate crate.) Many possible file formats are supported, including
<code>json</code>, <code>toml</code>, <code>yaml</code>, <code>hjson</code>, <code>ron</code>.
<code>Yaml</code> files are used currently in this example.

The order of precedence for configuration sources is:
1. Base configuration either explicitly specified by the <code>-c|--config</code> option or
   a <code>application</code> configuration file found in the <code>resources</code> directory
   under the current working directory.
2. Environment specific overrides (for <code>local</code> or <code>production</code>) identified
   via the <code>APP_ENVIRONMENT</code> environment variable. This can be used to easily support
   different properties required for development and production; e.g., for database and application server
   <code>host</code> and <code>port</code> properties.
3. An optional secrets file is supported so you can avoid storing passwords and other secret
   information in your code repository. In practice, a CI pipeline would source secrets from a
   secure repository (e.g., a highly-restricted git repository or something like Vault) and included
   in the <code>server</code>'s deployment. For the <code>loader</code>, the user could specify a local file.
4. Finally, environment variables can be used to override configuration settings. They must
   begin with <code>APP__</code> and path elements separated by a <code>__</code> delimiter.