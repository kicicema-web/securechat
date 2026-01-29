package com.securechat.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Send
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.securechat.viewmodel.ChatViewModel
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChatScreen(
    viewModel: ChatViewModel,
    onNavigateToAuth: () -> Unit
) {
    val conversations by viewModel.conversations.collectAsState()
    val messages by viewModel.messages.collectAsState()
    val currentConversation by viewModel.currentConversation.collectAsState()
    val currentContact by viewModel.currentContact.collectAsState()
    val userProfile by viewModel.userProfile.collectAsState()

    var showNewContactDialog by remember { mutableStateOf(false) }
    var messageText by remember { mutableStateOf("") }

    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val scope = rememberCoroutineScope()

    ModalNavigationDrawer(
        drawerState = drawerState,
        drawerContent = {
            ConversationDrawer(
                conversations = conversations,
                userProfile = userProfile,
                onConversationClick = { conv ->
                    viewModel.selectConversation(conv)
                    scope.launch { drawerState.close() }
                },
                onNewContactClick = { showNewContactDialog = true }
            )
        }
    ) {
        Scaffold(
            topBar = {
                if (currentConversation != null) {
                    TopAppBar(
                        title = {
                            Column {
                                Text(
                                    text = currentContact?.display_name ?: "Unknown",
                                    style = MaterialTheme.typography.titleMedium
                                )
                                Text(
                                    text = "🔒 Encrypted",
                                    style = MaterialTheme.typography.bodySmall.copy(
                                        color = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                )
                            }
                        },
                        navigationIcon = {
                            IconButton(onClick = { scope.launch { drawerState.open() } }) {
                                Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                            }
                        },
                        actions = {
                            IconButton(onClick = { /* TODO */ }) {
                                Icon(Icons.Default.MoreVert, contentDescription = "More")
                            }
                        }
                    )
                } else {
                    TopAppBar(
                        title = { Text("SecureChat") },
                        navigationIcon = {
                            IconButton(onClick = { scope.launch { drawerState.open() } }) {
                                Text("☰")
                            }
                        }
                    )
                }
            },
            bottomBar = {
                if (currentConversation != null) {
                    ChatInputBar(
                        value = messageText,
                        onValueChange = { messageText = it },
                        onSend = {
                            if (messageText.isNotBlank()) {
                                viewModel.sendMessage(messageText)
                                messageText = ""
                            }
                        }
                    )
                }
            }
        ) { padding ->
            if (currentConversation == null) {
                // Empty state
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(padding),
                    contentAlignment = Alignment.Center
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Text(
                            text = "💬",
                            style = MaterialTheme.typography.displayLarge
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        Text(
                            text = "Select a conversation",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            } else {
                // Messages list
                MessagesList(
                    messages = messages,
                    modifier = Modifier.padding(padding)
                )
            }
        }
    }

    if (showNewContactDialog) {
        NewContactDialog(
            onDismiss = { showNewContactDialog = false },
            onAdd = { name, key ->
                viewModel.addContact(name, key)
                showNewContactDialog = false
            },
            myPublicKey = viewModel.getPublicKey()
        )
    }
}

@Composable
fun ConversationDrawer(
    conversations: List<com.securechat.protocol.Conversation>,
    userProfile: com.securechat.protocol.UserProfile?,
    onConversationClick: (com.securechat.protocol.Conversation) -> Unit,
    onNewContactClick: () -> Unit
) {
    ModalDrawerSheet {
        Column(modifier = Modifier.fillMaxHeight()) {
            // User header
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(MaterialTheme.colorScheme.surfaceVariant)
                    .padding(16.dp)
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Box(
                        modifier = Modifier
                            .size(48.dp)
                            .background(
                                MaterialTheme.colorScheme.primary,
                                CircleShape
                            ),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = userProfile?.display_name?.firstOrNull()?.uppercase() ?: "U",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onPrimary
                        )
                    }
                    Spacer(modifier = Modifier.width(12.dp))
                    Column {
                        Text(
                            text = userProfile?.display_name ?: "User",
                            style = MaterialTheme.typography.titleMedium
                        )
                        Text(
                            text = "🔒 Encrypted",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }

            // Search bar
            OutlinedTextField(
                value = "",
                onValueChange = {},
                placeholder = { Text("Search conversations...") },
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                singleLine = true
            )

            // New contact button
            TextButton(
                onClick = onNewContactClick,
                modifier = Modifier.padding(horizontal = 16.dp)
            ) {
                Icon(Icons.Default.Add, contentDescription = null)
                Spacer(modifier = Modifier.width(8.dp))
                Text("New Contact")
            }

            Divider()

            // Conversations list
            LazyColumn {
                items(conversations) { conv ->
                    ConversationItem(
                        conversation = conv,
                        onClick = { onConversationClick(conv) }
                    )
                }
            }
        }
    }
}

@Composable
fun ConversationItem(
    conversation: com.securechat.protocol.Conversation,
    onClick: () -> Unit
) {
    ListItem(
        headlineContent = {
            Text(
                text = conversation.contact_id.take(8), // TODO: Get actual name
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        },
        supportingContent = {
            Text(
                text = conversation.last_message_preview ?: "No messages",
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.bodyMedium.copy(
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            )
        },
        leadingContent = {
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .background(
                        MaterialTheme.colorScheme.surfaceVariant,
                        CircleShape
                    ),
                contentAlignment = Alignment.Center
            ) {
                Text(conversation.contact_id.first().uppercase())
            }
        },
        trailingContent = if (conversation.unread_count > 0) {
            {
                Badge { Text(conversation.unread_count.toString()) }
            }
        } else null,
        modifier = Modifier.clickable(onClick = onClick)
    )
}

@Composable
fun MessagesList(
    messages: List<com.securechat.protocol.LocalMessage>,
    modifier: Modifier = Modifier
) {
    val listState = rememberLazyListState()

    LazyColumn(
        state = listState,
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 16.dp),
        reverseLayout = true
    ) {
        items(messages.reversed()) { message ->
            MessageBubble(message = message)
            Spacer(modifier = Modifier.height(8.dp))
        }
    }

    // Scroll to bottom on new messages
    LaunchedEffect(messages.size) {
        if (messages.isNotEmpty()) {
            listState.scrollToItem(0)
        }
    }
}

@Composable
fun MessageBubble(message: com.securechat.protocol.LocalMessage) {
    val isOutgoing = message.is_outgoing
    val backgroundColor = if (isOutgoing) {
        MaterialTheme.colorScheme.primary
    } else {
        MaterialTheme.colorScheme.surfaceVariant
    }
    val textColor = if (isOutgoing) {
        MaterialTheme.colorScheme.onPrimary
    } else {
        MaterialTheme.colorScheme.onSurfaceVariant
    }

    Box(
        modifier = Modifier.fillMaxWidth(),
        contentAlignment = if (isOutgoing) Alignment.CenterEnd else Alignment.CenterStart
    ) {
        Column(
            modifier = Modifier
                .widthIn(max = 280.dp)
                .background(
                    backgroundColor,
                    RoundedCornerShape(
                        topStart = 16.dp,
                        topEnd = 16.dp,
                        bottomStart = if (isOutgoing) 16.dp else 4.dp,
                        bottomEnd = if (isOutgoing) 4.dp else 16.dp
                    )
                )
                .padding(12.dp)
        ) {
            Text(
                text = message.content.toString(), // TODO: Format properly
                color = textColor,
                style = MaterialTheme.typography.bodyMedium
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = formatTime(message.timestamp),
                color = textColor.copy(alpha = 0.7f),
                style = MaterialTheme.typography.bodySmall
            )
        }
    }
}

@Composable
fun ChatInputBar(
    value: String,
    onValueChange: (String) -> Unit,
    onSend: () -> Unit
) {
    Surface(
        tonalElevation = 2.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            OutlinedTextField(
                value = value,
                onValueChange = onValueChange,
                placeholder = { Text("Type a message...") },
                modifier = Modifier.weight(1f),
                maxLines = 4,
                keyboardOptions = androidx.compose.foundation.text.KeyboardOptions(
                    imeAction = androidx.compose.ui.text.input.ImeAction.Send
                ),
                keyboardActions = androidx.compose.foundation.text.KeyboardActions(
                    onSend = { onSend() }
                )
            )
            Spacer(modifier = Modifier.width(8.dp))
            IconButton(
                onClick = onSend,
                enabled = value.isNotBlank()
            ) {
                Icon(
                    Icons.Default.Send,
                    contentDescription = "Send",
                    tint = MaterialTheme.colorScheme.primary
                )
            }
        }
    }
}

@Composable
fun NewContactDialog(
    onDismiss: () -> Unit,
    onAdd: (String, String) -> Unit,
    myPublicKey: String
) {
    var name by remember { mutableStateOf("") }
    var publicKey by remember { mutableStateOf("") }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Add New Contact") },
        text = {
            Column {
                Text(
                    "Your Public Key (share this):",
                    style = MaterialTheme.typography.labelMedium
                )
                SelectionContainer {
                    Text(
                        myPublicKey.take(40) + "...",
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.padding(vertical = 8.dp)
                    )
                }
                Divider(modifier = Modifier.padding(vertical = 16.dp))
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Contact Name") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = publicKey,
                    onValueChange = { publicKey = it },
                    label = { Text("Contact Public Key") },
                    minLines = 3,
                    maxLines = 3,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        },
        confirmButton = {
            TextButton(
                onClick = { onAdd(name, publicKey) },
                enabled = name.isNotBlank() && publicKey.isNotBlank()
            ) {
                Text("Add")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Cancel")
            }
        }
    )
}

private fun formatTime(timestamp: kotlinx.datetime.Instant): String {
    // TODO: Implement proper time formatting
    return timestamp.toString().takeLast(5)
}
