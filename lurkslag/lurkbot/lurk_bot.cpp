// Run like this:  simple_client address port
// Results in argv ["./simple_client", "address", "port"]

#include<sys/socket.h>
#include<sys/types.h>
#include<netinet/ip.h>
#include<netdb.h>
#include<arpa/inet.h>
#include<stdio.h>
#include<unistd.h>
#include<stdlib.h>
#include<string.h>
#include<iostream>
#include<vector>
using namespace std;

struct lurk_connection_message {
	uint8_t type = 13;
	uint16_t roomnumber;
	char name[32];
	uint16_t description_length;
	char *description = 0;

	~lurk_connection_message(){
		if(description){
			free(description);
			description = 0;
		}
	}
	bool read(int skt){
		ssize_t readlen;
		readlen = recv(skt, &type, 1, MSG_PEEK);
		if(readlen < 1 || type != 13){
			printf("Failed to read connection\n");
			return false;
		}
		ssize_t header_size = sizeof(struct lurk_connection_message) - sizeof(char*);
		if(recv(skt, this, header_size, MSG_WAITALL) != header_size)
			return false;
		if(description)
			free(description);
		description = (char*)malloc(description_length);
		if(recv(skt, description, description_length, MSG_WAITALL) != description_length)
			return false;
		return true;
	}
} __attribute__((packed));


struct lurk_character_message {
	uint8_t type = 10;
	uint8_t flags = 0xf8;
	char name[32];
	uint16_t attack, defense, regen;
	int16_t health;
	uint16_t gold, roomnumber, description_length;
	char *description = 0;
	
	void set_description(const char* d){
		description_length = strlen(d);
		description = (char*)malloc(description_length + 1);
		strncpy(description, d, description_length);
	}

	~lurk_character_message(){
		if(description){
			free(description);
			description = 0;
		}
	}

	bool write(int skt){
		ssize_t expected_size = sizeof(struct lurk_character_message) - sizeof(char*);
		if(send(skt, this, expected_size, 0) != expected_size)
			return false;
		return send(skt, description, description_length, 0) == description_length;
	}

	bool read(int skt){
		ssize_t readlen;
		readlen = recv(skt, &type, 1, MSG_PEEK);
		if(readlen < 1 || type != 10){
			printf("Failed to read character\n");
			return false;
		}
		ssize_t header_size = sizeof(struct lurk_character_message) - sizeof(char*);
		if(recv(skt, this, header_size, MSG_WAITALL) != header_size)
			return false;
		description = (char*)malloc(description_length);
		if(recv(skt, description, description_length, MSG_WAITALL) != description_length)
			return false;
		return true;
	}
} __attribute__((packed));

struct lurk_version_message {
	uint8_t type, major, minor;
	uint16_t extension_length;

	bool read(int skt){
		ssize_t readlen;
		readlen = recv(skt, &type, 1, MSG_PEEK);
		if(readlen < 1 || type != 14){
			printf("Failed to read version\n");
			return false;
		}
		readlen = recv(skt, &type, 5, MSG_WAITALL);
		return readlen == 5;
	}
} __attribute__((packed));

ostream& operator<<(ostream& out, const lurk_version_message &lvm){
	out << "Type:  " << (int)lvm.type << endl;
	out << "Version:  " << (int)lvm.major << "." << (int)lvm.minor << endl;
	out << "Extension Length:  " << lvm.extension_length << endl;
	return out;
}

struct lurk_game_message {
	uint8_t type;
	uint16_t initial_points, stat_limit, description_length;
	char *description = 0;

	bool read(int skt) {
		ssize_t readlen = recv(skt, this, 7, MSG_WAITALL);
		if(readlen < 7){
			printf("Failed to read first 7 bytes, giving up!\n");
			return false;
		}

		description = (char*)malloc(description_length + 1);
		description[description_length] = 0;
		readlen = recv(skt, description, description_length, MSG_WAITALL);
		if(readlen != description_length){
			printf("Failed to read description, giving up\n");
			free(description);
			description = 0;
			return false;
		}
		return true;
	}

	~lurk_game_message(){
		if(description)
			free(description);
	}
} __attribute__((packed)); // Otherwise the compiler will leave padding

ostream& operator<<(ostream& out, const lurk_game_message &lgm){
	out << "Type:  " << (int)lgm.type << endl;
	out << "Initial Points:  " <<  lgm.initial_points << endl;
	out << "Stat Limit:  " << lgm.stat_limit << endl;
	out << "Description Length:  " << lgm.description_length << endl;
	out << "Description:  " << lgm.description << endl;
	return out;
}

char general_buffer[65535];
bool lurk_ignorer(int skt){
	// Ignore list
	char next;
	recv(skt, &next, 1, MSG_PEEK);
	if(next == 1) { // Ignore a message
		recv(skt, general_buffer, 67, MSG_WAITALL);
		uint16_t description_length = *((uint16_t*)(general_buffer + 1));
		recv(skt, general_buffer, description_length, MSG_WAITALL);
	} else if (next == 7) { // Ignore an error
		recv(skt, general_buffer, 4, MSG_WAITALL);
		uint16_t description_length = *((uint16_t*)(general_buffer + 2));
		recv(skt, general_buffer, description_length, MSG_WAITALL);
	} else if (next == 8) { // Ignore an accept
		recv(skt, general_buffer, 2, MSG_WAITALL);
	} else if (next == 9) {	 // Ignore a room
		recv(skt, general_buffer, 37, MSG_WAITALL);
		uint16_t description_length = *((uint16_t*)(general_buffer + 35));
		recv(skt, general_buffer, description_length, MSG_WAITALL);
	}
	return false;
}

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
	// New assumption:  We don't know if the server will send a version or not!
	struct lurk_version_message lvm;
	if(lvm.read(skt))
		cout << lvm << endl;
	else
		cout << "No version sent\n";

	struct lurk_game_message lgm;
	lgm.read(skt);
	cout << lgm << endl;

	struct lurk_character_message lcm;
	lcm.attack = lgm.initial_points/3;
	lcm.defense = lgm.initial_points/3;
	lcm.regen = lgm.initial_points/3;
	strcpy(lcm.name, "A Bot");
	if(!lcm.write(skt)){
		printf("Failed to send character\n");
		return 1;
	}
	char start = 6;
	write(skt, &start, 1);
	
	/* What could the server send?
 	 * Character (10), Accept (8), Error (7), Room (9), Connection (13), Message (1) */
	struct lurk_connection_message new_connection;
	while(true){
		vector<int> connections;
		while(lurk_ignorer(skt));
		while(new_connection.read(skt))
			connections.push_back(new_connection.roomnumber);
		/* TODO:  Revisit logic here */
		printf("Connections:  ");
		for(int i : connections)
			printf("%d ", i);
		puts("");
		if(lcm.read(skt))
			printf("Just read character %s in room %d\n", lcm.name, lcm.roomnumber);
		uint8_t changeroom = 2;
		write(skt, &changeroom, 1);
		uint16_t rnum = connections[0];
		write(skt, &rnum, 2);
	}
	

	close(skt);
	return 0;
}

