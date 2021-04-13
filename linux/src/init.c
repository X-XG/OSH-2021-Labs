#include <stdio.h>
#include <stdlib.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/sysmacros.h>
#include <sys/wait.h>

int main(){
    pid_t pid;
    char * const argv[] = {NULL};

    pid = fork();
    if (pid == 0){
        execv("./1", argv);
        return 0;
    }
    waitpid(pid, NULL, 0);

    pid = fork();
    if (pid == 0){
        if (mknod("/dev/ttyS0", S_IFCHR | S_IRUSR | S_IWUSR, makedev(4, 64)) == -1){
            perror("mknod() failed");
            return -1;
        }
        if (mknod("/dev/ttyAMA0", S_IFCHR | S_IRUSR | S_IWUSR, makedev(204, 64)) == -1){
            perror("mknod() failed");
            return -1;
        }
        execv("./2", argv);
        return 0;
    }
    waitpid(pid, NULL, 0);

    pid = fork();
    if (pid == 0){
        if (mknod("/dev/fb0", S_IFCHR | S_IRUSR | S_IWUSR, makedev(29, 0)) == -1){
            perror("mknod() failed");
            return -1;
        }
        execv("./3", argv);
        return 0;
    }
    waitpid(pid, NULL, 0);

    while (1);
}
