// Run like this:  simple_client address port
// Results in argv ["./simple_client", "address", "port"]

#include <stdint.h>
#include<sys/socket.h>
#include<sys/types.h>
#include<netinet/ip.h>
#include<netdb.h>
#include<arpa/inet.h>
#include<stdio.h>
#include<unistd.h>
#include<stdlib.h>

int main(int argc, char ** argv){
	if(argc < 3){
		printf("Usage:  %s hostname port\n", argv[0]);
		return 1;
	}
	struct sockaddr_in sad;
	sad.sin_port = htons(atoi(argv[2]));
	sad.sin_family = AF_INET;

	int skt = socket(AF_INET, SOCK_STREAM, 0);

	// do a dns lookup
	struct hostent* entry = gethostbyname(argv[1]);
	if(!entry){
		if(h_errno == HOST_NOT_FOUND){
			printf("This is our own message that says the host wasn't found\n");
		}
		herror("gethostbyname");
		return 1;
	}
	struct in_addr **addr_list = (struct in_addr**)entry->h_addr_list;
	struct in_addr* c_addr = addr_list[0];
	char* ip_string = inet_ntoa(*c_addr);
	sad.sin_addr = *c_addr; // copy the address we found into sad
	// Finally done with DNS!
	printf("Connecting to:  %s\n", ip_string);

	if( connect(skt, (struct sockaddr*)&sad, sizeof(struct sockaddr_in)) ){
		perror("connect");
		return 1;
	}

  // read 5 int32's, then reverse endiannes with htonl
  for(int i = 0; i < 5; i++){
    int32_t curr;
    read(skt, &curr, 4);
    printf("int32_t #%d: %d\n", i, htonl(curr));
  }

	close(skt);
	return 0;
}

