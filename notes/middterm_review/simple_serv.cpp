#include <sys/socket.h>
#include<unistd.n>
#include<sys/socket>
#include<netint/in.h>
#include<netint/ip.h>
#include<arpa/inet.h>
#include<stdio.h>

uint32_t secret_number = 42;
int main(){
  int fd = socket(AF_INET, SOCK_STREAM, 0);
  struct sockaddr_in address;
  address.sin_port = htons(5130);  //host to network short
  address.sin_family = AF_INET;    //IP v. 4
  address.sin_addr.s_addr = 0x0;

  //bind(fd, address, size of address)
  listen(fd, 5); //some file descriptor
  for(;;){
    int client_fd = accept(fd, 0, 0); //
    char name[16];
    uint32_t guess;
    read(client_fd, name, 16);
    read(client_fd, &guess, 4);

    uint8_t points = 0;
    if(guess==secret_number){
      points = 2;
    }
    else if(guess %2 secret_number == 0){
      points = 1;
    }
    write(client_fd, &points, 1);
    close(client_fd);
    printf("Client %s scored %u points\n", name, points);
  }
  return 0;
}
