#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <pthread.h>
#define BUF 1000

struct Pipe {
    int fd_send;
    int fd_recv;
};

void *handle_chat(void *data) {
    struct Pipe *pipe = (struct Pipe *)data;
    char msg[1050000]="Message:";
    char buffer[BUF];
    ssize_t len;
    int start = 8;
    while(1){
        //memset(buffer,'\0',1024*sizeof(char));
        len = recv(pipe->fd_send, buffer, BUF-12, 0);
        printf("the len is: %ld\n",len);
        printf("the buffer is: %s////////////////////////",buffer);
        int i;
        int sig = 0;
        int num = 0;
        for (i=0;i<len;i++){
            if(buffer[i]=='\n'){
                num = i -sig +1;
                strncpy(msg+start,buffer+sig,num);
                
                //printf("the msg is:\n%s////////////\n",msg);
                //printf("start+sig is: %d\n",start+sig);
                send(pipe->fd_recv, msg, start+num, 0);
                sig=i+1;
                start =8;
            }
        }
        printf("sig is: %d\n",sig);
        if(sig!=len){
            num = len-sig;
            strncpy(msg+start,buffer+sig,num);
            start=start+num;
            printf("the start is: %d\n",start);
        }
    }
    return NULL;
}

int main(int argc, char **argv) {
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
    if (bind(fd, (struct sockaddr *)&addr, sizeof(addr))) {
        perror("bind");
        return 1;
    }
    if (listen(fd, 2)) {
        perror("listen");
        return 1;
    }
    int fd1 = accept(fd, NULL, NULL);
    int fd2 = accept(fd, NULL, NULL);
    if (fd1 == -1 || fd2 == -1) {
        perror("accept");
        return 1;
    }
    pthread_t thread1, thread2;
    struct Pipe pipe1;
    struct Pipe pipe2;
    pipe1.fd_send = fd1;
    pipe1.fd_recv = fd2;
    pipe2.fd_send = fd2;
    pipe2.fd_recv = fd1;
    pthread_create(&thread1, NULL, handle_chat, (void *)&pipe1);
    pthread_create(&thread2, NULL, handle_chat, (void *)&pipe2);
    pthread_join(thread1, NULL);
    pthread_join(thread2, NULL);
    return 0;
}