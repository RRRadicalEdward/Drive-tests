[![Build Status](https://travis-ci.org/RRRadicalEdward/Drive-tests.svg?branch=master)](https://travis-ci.org/RRRadicalEdward/Drive-tests)
[![Build Status](https://travis-ci.org/RRRadicalEdward/Drive-tests.svg?branch=backend)](https://travis-ci.org/RRRadicalEdward/Drive-tests)
# Rust REST API - Drive-tests
Driving tests site where frontend is written in HTML, CSS, JS and jQuery and backend in Rust by using Actix-web and Diesel


## API

### Address: **`localhost:5050`** 

### `POST /user` - registry a new user
```bash
curl -X POST 'https://localhost:5050/user' \
-H 'Content-Type: application/json' \
--data-raw '{
    "name": "sasha",
    "second_name": "yusuk",
    "password": "mypassword"
}'
```
- Request body 
```
{
    "name": string,
    "second_name": string,
    "password": string
}
```

- Response 
  - 200 Ok
    ```
    {
        "name": string,
        "second_name": string,
        "scores": 0
    }
    ```

### `GET /user` - returns data for the user
```bash
curl -X GET 'https://localhost:5050/user' \
-H 'Content-Type: application/json' \
--data-raw '{
    "name": "sasha",
    "second_name": "yusuk",
    "password": "mypassword"
}'
```
- Response
    - 302 Found
    ```
    {
        "name": string,
        "second_name": string,
        "scores": int
    }
    ```
    - 403 Forbidden - bad password
    - 404 Not Found - the user doesn't exists
    - 500 Internal Server Error - something bad happened on the server side, try to do the request again
    
### `GET /test` - returns a random test
```bash 
curl -X GET 'https://localhost:5050/test'
```
 - Response 
    - 200 Ok 
    ```
    {
		"id": int,
		"description": string,
		"answers": vec<string>,
		"image": string, //can be null, the string in base64
    }
    ```
    - 500 Internal Server Error - something bad happened on the server side, try to do the request again

### `GET /check_answer?test_id&answer_id` - check a test answer
```bash
curl -X GET 'https://localhost:5050/check_answer?test_id={int}&answer_id={int}'
```
  - Response
    - 200 Ok 
    ```
    {
		"description": string,
		"scores": int,
    }
    ```
    - 500 Internal Server Error - something bad happened on the server side
    
### `POST /check_test` - check a test answer with user data and if user has passed a test it will save the new scores
```bash
curl -X POST 'https://localhost:5050/check_test' \
-H 'Content-Type: application/json' \
--data-raw '{
    "answer": {
        "test_id": 17,
        "answer_id": 5
    },
    "user": {
        "name": "sasha",
        "second_name": "yusuk",
        "password": "mypassword"
    }
}'
```
- Request body 
```
{
    "answer": {
        "test_id": int,
        "answer_id": int
    },
    "user": {
        "name": string,
        "second_name": string,
        "password": string
    }
}
```
 - Response
    - 200 Ok 
    ```
    {
	    "description": string,
	    "scores": int,
    }
    ```
    - 403 Forbidden - bad password
    - 404 Not Found - the user doesn't exists
    - 500 Internal Server Error - something bad happened on the server side, try to do the request again

### `GET /healthy` - testing request to check if server is running
```bash 
curl -X GET 'https://localhost:5050/healthy'
```
  - Response
    - 200 Ok 
    ```
    "Drive-tests is working and healthy"
    ```
    - 404 NotFound - The server isn't running
