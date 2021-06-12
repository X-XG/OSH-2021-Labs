#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <pthread.h>
#define BUF 1000
#define MAX_USERS 34

struct Info {
    int fd_recv;
    int myid;
};
int client[MAX_USERS];

pthread_mutex_t mutex = PTHREAD_MUTEX_INITIALIZER;
int used[MAX_USERS] = { 0 };

void* handle_chat(void* data) {
    struct Info* info = (struct Info*)data;
    char msg[1050000] = "Message:";
    char buffer[BUF];
    ssize_t len;
    int start = 8;

    while (1) {
        len = recv(info->fd_recv, buffer, BUF - 12, 0);
        if (len <= 0) {
            used[info->myid] = 0;
            pthread_mutex_unlock(&mutex);
            return 0;
        }

        int i, j;
        int sig = 0;
        int num = 0;
        for (i = 0;i < len;i++) {
            if (buffer[i] == '\n') {
                num = i - sig + 1;
                strncpy(msg + start, buffer + sig, num);

                pthread_mutex_lock(&mutex);
                for (j = 0;j < MAX_USERS;j++) {
                    if (used[j] && j != info->myid) {
                        send(client[j], msg, start + num, 0);
                    }
                }
                pthread_mutex_unlock(&mutex);

                sig = i + 1;
                start = 8;
            }
        }
        if (sig != len) {
            num = len - sig;
            strncpy(msg + start, buffer + sig, num);
            start = start + num;
        }
    }
    return NULL;
}

int main(int argc, char** argv) {
    int port = atoi(argv[1]);
    int fd;
    if ((fd = socket(AF_INET, SOCK_STREAM, 0)) == 0) {
        perror("socket");
        return 1;
    }

    struct sockaddr_in addr;
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(port);
    socklen_t addr_len = sizeof(addr);

    if (bind(fd, (struct sockaddr*)&addr, sizeof(addr))) {
        perror("bind");
        return 1;
    }
    if (listen(fd, MAX_USERS)) {
        perror("listen");
        return 1;
    }

    pthread_t thread[MAX_USERS];
    struct Info info[MAX_USERS];
    while (1) {
        int i;
        int tmp_client = accept(fd, NULL, NULL);
        if (tmp_client == -1) {
            perror("accept");
            return 1;
        }
        
        for (i = 0;i < MAX_USERS;i++) {
            if (used[i] == 0) {
                used[i] = 1;
                client[i] = tmp_client;
                info[i].fd_recv = tmp_client;
                info[i].myid = i;
                pthread_create(&thread[i], NULL, handle_chat, (void*)&info[i]);
                break;
            }
        }
        if (i == MAX_USERS) {
            perror("MAX_USERS");
            return 1;
        }

    }

    return 0;
}