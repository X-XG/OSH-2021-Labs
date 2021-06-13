#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/socket.h>
#include <sys/select.h>
#include <sys/time.h>
#include <netinet/in.h>
#define BUF 1000
#define MAX_MSG 1050000
#define MAX_USERS 34

int used[MAX_USERS] = { 0 };
int client[MAX_USERS] = { 0 };

char msg[MAX_MSG] = "";

void handle_chat(int myid) {
    char buffer[BUF];
    ssize_t len;
    int start = 8;

    sprintf(msg, "user %2d:", myid);

    while (1) {
        len = recv(client[myid], buffer, BUF - 12, 0);
        if (len <= 0) {
            used[myid] = 0;
            close(client[myid]);
            return;
        }

        int i, j;
        int sig = 0;
        int num = 0;
        for (i = 0;i < len;i++) {
            if (buffer[i] == '\n') {
                num = i - sig + 1;
                strncpy(msg + start, buffer + sig, num);

                for (j = 0;j < MAX_USERS;j++) {
                    if (used[j] && j != myid) {
                        int remain = strlen(msg);
                        int sended = 0;;
                        while (remain > 0) {
                            sended = send(client[j], msg, start + num, 0);
                            if (sended == -1) {
                                perror("send");
                                exit(-1);
                            }
                            remain -= sended;
                        }
                    }
                }

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
    return;
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

    int i;
    int supfd = fd;
    fd_set fds;
    while (1) {
        FD_ZERO(&fds);
        FD_SET(fd, &fds);
        for (i = 0;i < MAX_USERS;i++) {
            if (used[i])FD_SET(client[i], &fds);
        }
        
        if (select(supfd + 1, &fds, NULL, NULL, NULL) > 0) {
            if (FD_ISSET(fd, &fds)) {
                int new_client = accept(fd, NULL, NULL);
                if (new_client == -1) {
                    perror("accept");
                    return 1;
                }

                fcntl(new_client, F_SETFL, fcntl(new_client, F_GETFL, 0) | O_NONBLOCK);
                
                if (supfd < new_client) {
                    supfd = new_client;
                }

                for (i = 0;i < MAX_USERS;i++) {
                    if (!used[i]) {
                        used[i] = 1;
                        client[i] = new_client;
                        break;
                    }
                }
            }
            else {
                for (i = 0;i < MAX_USERS;i++) {
                    if (used[i] && FD_ISSET(client[i], &fds)) {
                        handle_chat(i);
                    }
                }
            }
        }
    }

    return 0;
}