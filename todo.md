# HTTP Server To-Do List

## Default Error Pages

- [ ] Create default error pages for the following status codes:
  * - [ ] 400: Bad Request
  * - [ ] 403: Forbidden
  * - [ ] 404: Not Found
  * - [ ] 405: Method Not Allowed
  * - [ ] 413: Payload Too Large
  * - [ ] 500: Internal Server Error

## HTTP Request Handling

- [ ] GET Method
    * - [ ] Handle GET requests to serve static files.
    * - [ ] Return a default file (e.g., `index.html`) when the path is a directory.

- [ ] POST Method
    * - [ ] Handle POST requests to accept client data.
    * - [ ] Store data temporarily or process it accordingly.
    * - [ ] Enforce client body size limits.

- [ ] DELETE Method
    * - [ ] Handle DELETE requests to remove files or resources.
    * - [ ] Respond appropriately if the resource does not exist or cannot be deleted.

- [ ] Not Allowed Methods
    * - [ ] Return a 405 Method Not Allowed response for unsupported methods.

## Session and Cookies

- [ ] Session Management
    * - [ ] Generate a unique session ID for each user.
    * - [ ] Link sessions to cookies for client identification.

- [ ] Cookie Management
    * - [ ] Read cookies sent by the client.
    * - [ ] Set cookies in HTTP responses.

## Configuration File

- [ ] Hostname Handling
    * - [ ] Distinguish servers using the same IP and port but different hostnames.
    * - [ ] An error in one server's configuration doesn't bring down the entire server.

- [ ] Custom Error Pages
    * - [ ] Allow configuration of custom error pages for specific error codes.
    * - [ ] Test custom error pages with different scenarios.

- [ ] Default Directory File
    * - [ ] Serve a default file (e.g., index.html) when the requested path is a directory.

- [ ] Client Body Size Limit
    * - [ ] Reject requests exceeding the configured size limit with a 413 Payload Too Large error.

- [ ] Accepted Methods
    * - [ ] Define a list of accepted methods for each route.
    * - [ ] Return a 405 Method Not Allowed response for disallowed methods.

- [ ] Define HTTP redirections.

## Features to discuss about

- [ ] Testing the Server

- [ ] CGI Support