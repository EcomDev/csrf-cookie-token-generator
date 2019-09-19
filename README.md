CSRF Token Generator HTTP Server
================================

Usually generation of tokens for preventing CSRF attacks can be resource expensive as you have to initialize a whole application and store unique token in session storage between the requests. 

This HTTP server comes to tackle this issue by introducing cookie only based CSRF token. In response to HTTP request, server returns two cookies (cookie names are configurable): `token` and `checksum`. Token cookie is readable by JavaScript, buck checksum cookie is HTTP only cookie to be accessed by your backend application to verify the cookie signature.
This checksum is using secret key that you specify as a command argument for a server that it is not exposed to an outside world.

Installation
------------
During release of new version of the HTTP server a new binary for major *nix like OS are compiled and [can be found here](https://github.com/EcomDev/csrf-cookie-token-generator/releases/latest). You can also compile it from sources on a platform of your choice by using `cargo build` command.

After you downloaded or compiled a binary, you can run a tool with such options:

```
USAGE:
    csrf-cookie-token [OPTIONS] <checksum-secret>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --checksum-cookie <checksum-cookie-name>     [default: varnish_token_checksum]
    -d, --cookie-domain <cookie-domain>              [default: ]
    -p, --server-port <port>                         [default: 9999]
    -t, --token-cookie <token-cookie-name>           [default: varnish_token]

ARGS:
    <checksum-secret> 
```

After you've setup a server to run in the background, you can simply proxy calls for token generation by the url of your choice using this simple Nginx rule:

```
location /generate-csrf-token {
   proxy_pass http://127.0.0.1:9999;
   proxy_set_header Host $http_host;
}
```

The server always binds to 127.0.0.1 so you cannot access it without some kind of proxying. 

Verifying the generated token
-----------------------------

In order to implement the CSRF token verification you need to create first a JSON string with such data:

```json
{"salt":"[your_secret_key]","token":"[csrf_token_send_by_user]"}
```

Then you just hash whole string with `md5` (yeah, I know it is not the strongest one, but for a short lived checksum is more than enough). Make sure JSON string is not pretty printed and does not have any whitespaces.

After you verified the token in you application, you **MUST** regenerate a token and checksum and set appropriate cookies yourself, to make sure if key is leaked by any other means it cannot be re-used.


