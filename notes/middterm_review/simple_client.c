#include<unistd.h>
#include<sys/socket.h>
#include<netinet/ip.h>
#include<netinet/in.h>
#include<stdio.h>
#include<stdlib.h>
#include<arpa/inet.h>

//usage: client name guess
int main(int argc, char** argv){
  //socket
  //connect
  struct sockaddr_in address;
  address.sin_family = AF_INET;
  address.sin_port = htons(5130);
  //inet_ntoa 
  //could use hex
  //0x0F000001;
  //DNS lookup has been most popular on tests
  address.sin_addr.s_addr = 0x0100007F;// do we want to do a DNS lookup or do we want to hard-code the IP here?
  printf("Connecting to %s\n", inet_ntoa(address.sin_addr));

  //inet socket, tcp socket
  int fd = socket(AF_INET, SOCK_STREAM, 0);
  connect(fd, (struct sockadder*)(&address), sizeof(address));

  uint32_t guess = atoi(argv[2]);
  write(fd, argv[1], 16);
  write(fd, &guess, 4);
  uint8_t points;
  read(fd, &points, 1);
  printf("Score: %u points\n", points);
  return 0;
}
