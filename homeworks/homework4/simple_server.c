#include<sys/socket.h>
#include<sys/types.h>
#include<netinet/ip.h>
#include<arpa/inet.h>
#include<stdio.h>
#include<unistd.h>
#include<stdlib.h>

int main(int argc, char ** argv){
	struct sockaddr_in sad;
	sad.sin_port = htons(5143);
	sad.sin_addr.s_addr = INADDR_ANY;
	sad.sin_family = AF_INET;

	int skt = socket(AF_INET, SOCK_STREAM, 0); // Step 1
	if(skt == -1){
		perror("socket");
		return 1;
	}
	if( bind(skt, (struct sockaddr*)(&sad), sizeof(struct sockaddr_in)) ){ // step 2
		perror("bind");
		return 1;
	}
	if( listen(skt, 5) ){ // step 3
		perror("listen");
		return 1;
	}	
	while(1){
		int client_fd;
		struct sockaddr_in client_address;
		socklen_t address_size = sizeof(struct sockaddr_in);
		client_fd = accept(skt, (struct sockaddr *)(&client_address), &address_size); // step 4
		write(client_fd, "Good Morning ", 13);
		write(client_fd, "Now it is afternoon", 20);
		char client_message[100];
		read(client_fd, client_message, 100);
		printf("Client sent:  %s\n", client_message);
		printf("Connection made from address %s\n", inet_ntoa(client_address.sin_addr));
		close(client_fd);
	}
	
	return 0;
}
