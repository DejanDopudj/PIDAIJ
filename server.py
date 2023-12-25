import socket
import threading
from concurrent.futures import ThreadPoolExecutor
import json
import time

class DynamicThreadPoolServer:
    def __init__(self, host, port, initial_threads=5, max_threads=20, backlog_threshold=10):
        self.host = host
        self.port = port
        self.initial_threads = initial_threads
        self.max_threads = max_threads
        self.backlog_threshold = backlog_threshold
        self.dictionary = {}
        self.lock = threading.Lock()
        self.executor = None
        self.i = 0
        self.start_time = None
    def start_server(self):
        server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server_socket.bind((self.host, self.port))
        server_socket.listen(20000) 
        print("Server listening on {}:{}".format(self.host, self.port))

        self.executor = ThreadPoolExecutor(max_workers=self.initial_threads)
        while True:
            client_socket, address = server_socket.accept()
            self.executor.submit(self.handle_client, client_socket)

    def print_dict_info(self):
        for key, value in self.dictionary.items():
            print("Key: {}".format(key))
            print("Value: {}".format(value))
            print("Type of Value: {}".format(type(value)))
            print("------")

    def handle_client(self, client_socket):
        if self.i == 0:
            self.start_time = time.time()
        request = client_socket.recv(1024).decode('utf-8')
        if request.startswith("PUT"):
            body = request.split("\r\n\r\n")[1]
            data = json.loads(body)
            key = data["key"]
            value = data["value"]

            self.put(key, value)

            response_body = "PUT successful"
            status_line = "HTTP/1.1 200 OK"

        elif request.startswith("GET"):
            key = request.split("?")[1].split(" ")[0].split("=")[1]
            value = self.get(key)
            response_body = "GET result: {}".format(value)
            status_line = "HTTP/1.1 200 OK"

        else:
            response_body = "Invalid request"
            status_line = "HTTP/1.1 400 Bad Request"
        response_headers = "Content-Type: text/plain\r\n"
        response = "{}\r\n{}\r\n\r\n{}".format(status_line, response_headers, response_body)

        self.i+=1
        if self.i >= 10000:
            total_time = time.time() - self.start_time
            print(total_time)
        client_socket.send(response.encode('utf-8'))
        client_socket.close()



    def put(self, key, value):
        with self.lock:
            self.dictionary[key] = value

    def get(self, key):
        with self.lock:
            return self.dictionary.get(key, "NULL")

if __name__ == "__main__":
    server = DynamicThreadPoolServer("127.0.0.1", 8081, initial_threads=5, max_threads=12, backlog_threshold=10)
    server.start_server()
