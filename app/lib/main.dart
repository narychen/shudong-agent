import 'dart:convert';
import 'package:flutter/material.dart';

import 'rust_core.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'Shudong Agent',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF2F6BFF)),
        useMaterial3: true,
      ),
      home: const MyHomePage(),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key});

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  final TextEditingController _controller = TextEditingController();
  final TextEditingController _apiKeyController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  
  String _status = 'Initializing...';
  bool _busy = false;
  bool _initialized = false;
  String _selectedProvider = 'OpenAI';
  String _selectedModel = 'gpt-4';
  List<Map<String, String>> _messages = [];

  final Map<String, List<String>> _models = {
    'OpenAI': ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo'],
    'Anthropic': ['claude-3-opus-20240229', 'claude-3-sonnet-20240229', 'claude-3-haiku-20240307'],
    'Ollama': ['llama3', 'mistral', 'codellama'],
  };

  @override
  void initState() {
    super.initState();
    _checkStatus();
  }

  void _checkStatus() {
    try {
      final core = RustCore.instance;
      final status = jsonDecode(core.getStatus()) as Map<String, dynamic>;
      setState(() {
        _initialized = status['initialized'] == true;
        _status = _initialized ? 'Ready' : 'Not initialized';
      });
    } catch (e) {
      setState(() {
        _status = 'Rust core not wired: $e';
      });
    }
  }

  Future<void> _initializeAgent() async {
    if (_apiKeyController.text.trim().isEmpty) {
      setState(() {
        _status = 'Please enter API key';
      });
      return;
    }

    final config = {
      'provider': _selectedProvider,
      'api_key': _apiKeyController.text.trim(),
      'model': _selectedModel,
      'system_prompt': '你是一个专业的编码助手，擅长分析代码、提供优化建议、执行开发任务。请用中文回复。',
    };

    setState(() {
      _busy = true;
      _status = 'Initializing agent...';
    });

    try {
      final core = RustCore.instance;
      final result = core.init(config);
      setState(() {
        _initialized = result['success'] == true;
        _status = _initialized ? 'Agent initialized successfully' : 'Init failed: ${result['error']}';
        _busy = false;
      });
    } catch (error) {
      setState(() {
        _status = 'Error: $error';
        _busy = false;
      });
    }
  }

  Future<void> _sendTask() async {
    final task = _controller.text.trim();
    if (task.isEmpty || !_initialized) {
      setState(() {
        _status = _initialized ? 'Please type a task.' : 'Please initialize agent first.';
      });
      return;
    }

    setState(() {
      _busy = true;
      _messages.add({'role': 'user', 'content': task});
    });
    _controller.clear();

    try {
      final core = RustCore.instance;
      final result = core.processTask(task);
      
      if (result['success'] == true) {
        setState(() {
          _messages.add({'role': 'assistant', 'content': result['response']});
          _status = 'Ready';
        });
      } else {
        setState(() {
          _status = 'Error: ${result['error']}';
        });
      }
    } catch (error) {
      setState(() {
        _status = 'Error: $error';
      });
    } finally {
      setState(() {
        _busy = false;
      });
    }
    
    _scrollToBottom();
  }

  void _scrollToBottom() {
    Future.delayed(const Duration(milliseconds: 100), () {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
  }

  void _showSettings() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Agent Settings'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              DropdownButtonFormField<String>(
                value: _selectedProvider,
                decoration: const InputDecoration(labelText: 'LLM Provider'),
                items: _models.keys.map((provider) {
                  return DropdownMenuItem(
                    value: provider,
                    child: Text(provider),
                  );
                }).toList(),
                onChanged: (value) {
                  setState(() {
                    _selectedProvider = value!;
                    _selectedModel = _models[value]![0];
                  });
                },
              ),
              const SizedBox(height: 16),
              DropdownButtonFormField<String>(
                value: _selectedModel,
                decoration: const InputDecoration(labelText: 'Model'),
                items: _models[_selectedProvider]!.map((model) {
                  return DropdownMenuItem(
                    value: model,
                    child: Text(model),
                  );
                }).toList(),
                onChanged: (value) {
                  setState(() {
                    _selectedModel = value!;
                  });
                },
              ),
              const SizedBox(height: 16),
              TextField(
                controller: _apiKeyController,
                decoration: const InputDecoration(
                  labelText: 'API Key',
                  border: OutlineInputBorder(),
                ),
                obscureText: true,
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              Navigator.pop(context);
              _initializeAgent();
            },
            child: const Text('Initialize'),
          ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    _apiKeyController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Shudong Agent'),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: _showSettings,
          ),
        ],
      ),
      body: Column(
        children: [
          if (!_initialized)
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(16),
              margin: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Colors.amber.shade50,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.amber.shade200),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Setup Required',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 8),
                  const Text('Please configure your LLM provider and API key in settings.'),
                  const SizedBox(height: 16),
                  SizedBox(
                    width: double.infinity,
                    child: FilledButton(
                      onPressed: _showSettings,
                      child: const Text('Open Settings'),
                    ),
                  ),
                ],
              ),
            ),
          Expanded(
            child: _messages.isEmpty
                ? Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(
                          Icons.smart_toy_outlined,
                          size: 64,
                          color: Colors.grey.shade400,
                        ),
                        const SizedBox(height: 16),
                        Text(
                          'Send a task to the agent',
                          style: Theme.of(context).textTheme.titleMedium?.copyWith(
                                color: Colors.grey.shade600,
                              ),
                        ),
                      ],
                    ),
                  )
                : ListView.builder(
                    controller: _scrollController,
                    padding: const EdgeInsets.all(16),
                    itemCount: _messages.length,
                    itemBuilder: (context, index) {
                      final message = _messages[index];
                      final isUser = message['role'] == 'user';
                      return Align(
                        alignment: isUser ? Alignment.centerRight : Alignment.centerLeft,
                        child: Container(
                          constraints: const BoxConstraints(maxWidth: 400),
                          margin: const EdgeInsets.only(bottom: 12),
                          padding: const EdgeInsets.all(12),
                          decoration: BoxDecoration(
                            color: isUser
                                ? Theme.of(context).colorScheme.primaryContainer
                                : Theme.of(context).colorScheme.surfaceContainerHighest,
                            borderRadius: BorderRadius.circular(12),
                          ),
                          child: SelectableText(message['content']!),
                        ),
                      );
                    },
                  ),
          ),
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.05),
                  blurRadius: 10,
                  offset: const Offset(0, -2),
                ),
              ],
            ),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _controller,
                    minLines: 1,
                    maxLines: 4,
                    decoration: const InputDecoration(
                      border: OutlineInputBorder(),
                      hintText: 'Describe your task...',
                    ),
                    onSubmitted: (_) => _sendTask(),
                  ),
                ),
                const SizedBox(width: 12),
                FilledButton(
                  onPressed: (_busy || !_initialized) ? null : _sendTask,
                  child: _busy
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.send),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
